on run argv
  if (count of argv) is 0 then
    return
  end if

  set theTitle to item 1 of argv
  set theBody to ""
  if (count of argv) > 1 then
    set theBody to item 2 of argv
  end if

  display notification theBody with title theTitle
end run
