# 2hoge
### reinventing the music player

## motivation
nowadays, music listening is either done through streaming services or by listening to local files. both of these methods come with their own set of advantages and disadvantages. notably, they also are very hard to use together - your music library is always seperate between music sources. furthermore virtually all existing solutions lack social features (or only provide very basic ones). external websites like last.fm provide some social features but will also always be limited in features due to their external nature.

heres where 2hoge comes in. 2hoge is a fully plugin driven music player and music library management tool with social features that makes zero assumptions about your music sources and is built to support using multiple sources at once.

### music library
2hoges plugin driven nature lets you easily incorporate a variety of different music sources into your library - that playlist of good music you found on youtube? import it into your library, then search all of the songs from it up on soulseek and have high quality flacs from it available in minutes with minimal effort.

plugins can provide a variety of features, acting as a full music source, provider for song searches, extending your existing library with additional metadata or even allowing 2hoge to play new file formats

### social features
see what your friends are listening to, listen along, share music, host and join public and private listening parties and more!

social features use peer to peer technology where possible (most notably during file sharing), making the requirements for central servers very minimal. they basically only exist to help clients find each other and to cache user profile information while a user is offline. furthermore no single central server exists - users are able (and encouraged!) to host their own, users on different servers can still communicate with each other. all user data is stored locally which also makes it trivial for a user to switch to a different server.

social functionality is also fully extendable using plugins - the last.fm plugin for example lets you scrobble your tracks and even import and view your friends profiles from last.fm

### fearless plugin installs
plugins in 2hoge are fully self contained single file wasm binaries. this means theyre fully sandboxed and as a user you can be sure they only access the files and urls they claim to. this lets you confidently install plugins knowing they wont wreak havoc on your computer or send data to places you dont want it ending up at. 

as a plugin developer, wasm plugins mean you can write plugins in your language of choice as long as it is supported by [extism](https://extism.org/). at the time of writing this includes rust, javascript, go, c(++), zig, c#, haskell and assemblyscript!