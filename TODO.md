More features:

 ☐ HANDLE ERRORS
 ☐ Use Bot struct to group all commands together and do automatic command matching.
 ☐ /edit command to modify activities
   ☐ `/edit530 time 11:00`
   ☐ `/edit530 desc <new description>`
   ☐ `/edit530 activity <new activity shortcut>`
 ☐ (not needed then?) admin web interface to
   ☐ edit start time of planned activity
   ☐ change planned activity type
 ☐ admin interface (admin-only access) to
   ☐ find activities ids
       ☐ `/activities ids` similar to `/activites` but for actual activities, not shortcuts.
   ☐ add new activities w/ shortcuts
       ☐ `/activities add key=value,key=value` e.g.
           ☐ `/activities add min_fireteam_size=1,max_fireteam_size=6,name="Last Wish, Enhance",mode="prestige"` etc. <- note "," in name - should parse
       ☐ `/activities edit ACTIVITY_ID key=value,key=value`
       ☐ `/activities addsc SHORTCUT ACTIVITY_ID`
   ☐ change guardian PSN name
       ☐ `/editguar @alexundr psn Kayouga`
   ☐ edit guardian clan
       ☐ `/editguar Kayouga clan AEGL`
   ☐ other fields in guardians table
       ☐ `/editguar GUARDIAN_ID <field_name> <freeform value>`
       ☐ GUARDIAN_ID could be int, telegram name or psn name
   ☐ just show guardian fields
       ☐ `/editguar GUARDIAN_ID`
   ☐ manage admins (superadmin can add/remove admins, admins cannot add more admins?)
       ☐ `/manage` catch-all command for these things
           * `list-admins`, `add-admin`, `remove-admin` subcommands
 ☐ Track weekly raids cycle automatically,
   ☐ disallow creating weekly raids when they're inactive
   ☐ suggest next closest week when it's active
 ☐ Inline buttons to join or leave raid
 ☐ Interactive calendar + clock picker to plan raids
   ☐ this may require full actor framework already to track states properly
     ☐ actor per user creating raid?
 ☐ Rewrite with actors, self-healing and other nice-to-have things.
   - actix / riker could be used for structuring the bot as independent entities
    1. supervisor to restart failing telegram connection
       - see telegram-event-bot for structure idea
       - telecord uses a simplistic approach
    2. actix-diesel thingie to run blocking diesel in a separate actor
       - https://github.com/actix/examples/blob/f8e3570bd16bcf816070af20f210ce1b4f2fca69/diesel/src/main.rs#L64-L70
    3. futures-cpupool as a very primitive wrapper
       - https://github.com/diesel-rs/diesel/issues/399
    4. tokio threadpool blocking tasks
       - https://github.com/tokio-rs/tokio/pull/317
    5. bb8 as diesel pool
       - could this help?
 ☐ Honor TELEGRAM_BOT_TIMEZONE envvar, instead of hardcoded Europe/Moscow.

Remember to use `async fn`!!

Send function:
- https://github.com/bytesnake/telebot/blob/master/telebot-derive/src/lib.rs#L239

