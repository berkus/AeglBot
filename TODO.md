More features:

4. admin interface to
   - add activities / shortcuts
   - edit start time of planned activity
   - change planned activity type
5. Track weekly raids cycle automatically, 
   - disallow creating weekly raids when they're inactive
   - suggest next closest week when it's active
6. Inline buttons to join or leave raid
7. Interactive calendar + clock picker to plan raids
   - this may require full actor framework already to track states properly
     - actor per user creating raid?
8. Rewrite with actors, self-healing and other nice-to-have things.
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

Remember to use `async fn`!!

telebot has a command registration interface - see how flexible it is?


Send function:
- https://github.com/bytesnake/telebot/blob/master/telebot-derive/src/lib.rs#L239

