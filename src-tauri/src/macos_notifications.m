#import <Foundation/Foundation.h>
#import <UserNotifications/UserNotifications.h>
#include <dispatch/dispatch.h>
#include <stdlib.h>
#include <string.h>

@interface DragonClawNotificationDelegate : NSObject <UNUserNotificationCenterDelegate>
@end

@implementation DragonClawNotificationDelegate

- (void)userNotificationCenter:(UNUserNotificationCenter*)center
       willPresentNotification:(UNNotification*)notification
         withCompletionHandler:(void (^)(UNNotificationPresentationOptions options))completionHandler {
    if (completionHandler == nil) {
        return;
    }

    if (@available(macOS 11.0, *)) {
        completionHandler(
            UNNotificationPresentationOptionBanner | UNNotificationPresentationOptionList | UNNotificationPresentationOptionSound
        );
    } else {
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
        completionHandler(UNNotificationPresentationOptionAlert | UNNotificationPresentationOptionSound);
#pragma clang diagnostic pop
    }
}

- (void)userNotificationCenter:(UNUserNotificationCenter*)center
didReceiveNotificationResponse:(UNNotificationResponse*)response
         withCompletionHandler:(void (^)(void))completionHandler {
    if (completionHandler != nil) {
        completionHandler();
    }
}

@end

@interface DragonClawLegacyNotificationDelegate : NSObject <NSUserNotificationCenterDelegate>
@end

@implementation DragonClawLegacyNotificationDelegate

- (BOOL)userNotificationCenter:(NSUserNotificationCenter*)center shouldPresentNotification:(NSUserNotification*)notification {
    return YES;
}

@end

static DragonClawNotificationDelegate* dragonclaw_shared_delegate = nil;
static DragonClawLegacyNotificationDelegate* dragonclaw_shared_legacy_delegate = nil;

static void dragonclaw_set_error(char** error_out, NSString* message) {
    if (error_out == NULL) {
        return;
    }

    *error_out = NULL;
    if (message == nil || message.length == 0) {
        return;
    }

    const char* utf8 = message.UTF8String;
    if (utf8 == NULL) {
        return;
    }

    *error_out = strdup(utf8);
}

static void dragonclaw_ensure_delegate(void) {
    static dispatch_once_t once_token;
    dispatch_once(&once_token, ^{
        dragonclaw_shared_delegate = [DragonClawNotificationDelegate new];
    });

    [UNUserNotificationCenter currentNotificationCenter].delegate = dragonclaw_shared_delegate;
}

static BOOL dragonclaw_is_app_bundle_runtime(void) {
    NSURL* bundle_url = [NSBundle mainBundle].bundleURL;
    if (bundle_url == nil) {
        return NO;
    }

    NSString* path_extension = bundle_url.pathExtension;
    if (path_extension == nil) {
        return NO;
    }

    return [path_extension caseInsensitiveCompare:@"app"] == NSOrderedSame;
}

static void dragonclaw_ensure_legacy_delegate(void) {
    static dispatch_once_t once_token;
    dispatch_once(&once_token, ^{
        dragonclaw_shared_legacy_delegate = [DragonClawLegacyNotificationDelegate new];
    });

    [NSUserNotificationCenter defaultUserNotificationCenter].delegate = dragonclaw_shared_legacy_delegate;
}

static UNAuthorizationStatus dragonclaw_get_authorization_status(void) {
    __block UNAuthorizationStatus status = UNAuthorizationStatusNotDetermined;
    dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);

    [[UNUserNotificationCenter currentNotificationCenter]
        getNotificationSettingsWithCompletionHandler:^(UNNotificationSettings* settings) {
            status = settings.authorizationStatus;
            dispatch_semaphore_signal(semaphore);
        }];

    dispatch_semaphore_wait(semaphore, dispatch_time(DISPATCH_TIME_NOW, (int64_t)(2 * NSEC_PER_SEC)));
    return status;
}

static BOOL dragonclaw_request_authorization(char** error_out) {
    __block BOOL granted = NO;
    __block NSError* request_error = nil;
    dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);
    UNAuthorizationOptions options = UNAuthorizationOptionAlert | UNAuthorizationOptionBadge | UNAuthorizationOptionSound;

    [[UNUserNotificationCenter currentNotificationCenter]
        requestAuthorizationWithOptions:options
                      completionHandler:^(BOOL did_grant, NSError* error) {
                          granted = did_grant;
                          request_error = error;
                          dispatch_semaphore_signal(semaphore);
                      }];

    if (dispatch_semaphore_wait(semaphore, dispatch_time(DISPATCH_TIME_NOW, (int64_t)(5 * NSEC_PER_SEC))) != 0) {
        dragonclaw_set_error(error_out, @"等待系统通知权限响应超时。");
        return NO;
    }

    if (request_error != nil) {
        dragonclaw_set_error(
            error_out,
            [NSString stringWithFormat:@"申请系统通知权限失败：%@", request_error.localizedDescription ?: @"未知错误"]
        );
        return NO;
    }

    if (!granted) {
        dragonclaw_set_error(error_out, @"系统通知权限未授予，请在 macOS 系统设置的“通知”里允许当前应用发送通知。");
        return NO;
    }

    return YES;
}

int dragonclaw_show_user_notification(const char* title_utf8, const char* body_utf8, char** error_out) {
    @autoreleasepool {
        NSString* title = title_utf8 != NULL ? [NSString stringWithUTF8String:title_utf8] : @"";
        NSString* body = body_utf8 != NULL ? [NSString stringWithUTF8String:body_utf8] : @"";
        if (title == nil || title.length == 0) {
            dragonclaw_set_error(error_out, @"通知标题不能为空。");
            return 0;
        }

        if (!dragonclaw_is_app_bundle_runtime()) {
            dragonclaw_set_error(error_out, @"当前运行的是未打包的开发版，不能直接使用 macOS 原生通知中心。");
            return 0;
        }

        dragonclaw_ensure_delegate();

        UNAuthorizationStatus status = dragonclaw_get_authorization_status();
        if (status == UNAuthorizationStatusNotDetermined) {
            if (!dragonclaw_request_authorization(error_out)) {
                return 0;
            }
            status = dragonclaw_get_authorization_status();
        }

        if (status == UNAuthorizationStatusDenied) {
            dragonclaw_set_error(error_out, @"系统通知已被关闭，请在 macOS 系统设置的“通知”里为当前应用开启提醒。");
            return 0;
        }

        UNMutableNotificationContent* content = [UNMutableNotificationContent new];
        content.title = title;
        if (body != nil && body.length > 0) {
            content.body = body;
        }
        content.sound = [UNNotificationSound defaultSound];

        NSString* identifier = [[NSUUID UUID] UUIDString];
        UNTimeIntervalNotificationTrigger* trigger = [UNTimeIntervalNotificationTrigger triggerWithTimeInterval:0.1 repeats:NO];
        UNNotificationRequest* request = [UNNotificationRequest requestWithIdentifier:identifier content:content trigger:trigger];

        __block NSError* add_error = nil;
        dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);
        [[UNUserNotificationCenter currentNotificationCenter]
            addNotificationRequest:request
             withCompletionHandler:^(NSError* error) {
                 add_error = error;
                 dispatch_semaphore_signal(semaphore);
             }];

        if (dispatch_semaphore_wait(semaphore, dispatch_time(DISPATCH_TIME_NOW, (int64_t)(2 * NSEC_PER_SEC))) != 0) {
            dragonclaw_set_error(error_out, @"提交系统通知超时。");
            return 0;
        }

        if (add_error != nil) {
            dragonclaw_set_error(
                error_out,
                [NSString stringWithFormat:@"系统通知发送失败：%@", add_error.localizedDescription ?: @"未知错误"]
            );
            return 0;
        }

        return 1;
    }
}

void dragonclaw_free_c_string(char* value) {
    if (value != NULL) {
        free(value);
    }
}

int dragonclaw_show_legacy_user_notification(const char* title_utf8, const char* body_utf8, char** error_out) {
    @autoreleasepool {
        NSString* title = title_utf8 != NULL ? [NSString stringWithUTF8String:title_utf8] : @"";
        NSString* body = body_utf8 != NULL ? [NSString stringWithUTF8String:body_utf8] : @"";
        if (title == nil || title.length == 0) {
            dragonclaw_set_error(error_out, @"通知标题不能为空。");
            return 0;
        }

        dragonclaw_ensure_legacy_delegate();

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
        NSUserNotification* notification = [NSUserNotification new];
        notification.title = title;
        if (body != nil && body.length > 0) {
            notification.informativeText = body;
        }
        notification.soundName = NSUserNotificationDefaultSoundName;
        [[NSUserNotificationCenter defaultUserNotificationCenter] deliverNotification:notification];
#pragma clang diagnostic pop

        return 1;
    }
}
