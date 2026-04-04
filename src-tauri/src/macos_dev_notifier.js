function run(argv) {
    if (!argv || argv.length === 0) {
        return;
    }

    const title = String(argv[0] || "").trim();
    if (!title) {
        return;
    }

    const body = String(argv[1] || "");
    const app = Application.currentApplication();
    app.includeStandardAdditions = true;
    app.displayNotification(body, { withTitle: title });
}
