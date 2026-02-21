# VHS Demo: talkd — P2P messaging between two nodes

Output demo.gif

Require tmux

Set Shell "bash"
Set FontSize 36
Set Width 2560
Set Height 1440
Set Padding 30
Set Theme "Molokai"
Set TypingSpeed 50ms
Set CursorBlink false

# Setup tmux: Alice (left), Bob SSH (right)
# Both daemons start fresh, channel data cleared
Hide
  Type "tmux -f /dev/null -L test new-session -- /bin/bash" Enter
  Type "tmux split-window -d -h -- /bin/bash && \" Enter
  Type "tmux set status && \" Enter
  Type 'tmux setw pane-border-style "fg=0" && \' Enter
  Type 'tmux setw pane-active-border-style "fg=0"' Enter
  Sleep 0.5
  Type 'export PS1="alice $ "' Enter
  Sleep 0.3
  # Start Alice's daemon early so gossip has time to connect
  Type "talkd id > /dev/null" Enter
  Sleep 1
  Type "clear" Enter
  Sleep 0.3
  # Bob pane: SSH in, start daemon
  Ctrl+B
  Type "o"
  Sleep 0.3
  Type "ssh ukvps" Enter
  Sleep 3
  Type 'export PS1="bob $ "' Enter
  Sleep 0.3
  Type "talkd id > /dev/null" Enter
  Sleep 1
  Type "clear" Enter
  Sleep 0.3
  # Back to Alice, wait for gossip peers to connect
  Ctrl+B
  Type "o"
  Sleep 3
  Ctrl+L
  Sleep 0.5
Show

Sleep 1s

# === 1. Identity ===

# Alice: show ID
Type "talkd id"
Sleep 300ms
Enter
Sleep 3s

# Bob: show ID
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd id"
Sleep 300ms
Enter
Sleep 3s

# === 2. Add contacts ===

# Bob: add Alice
Type "talkd add alice b0434dda688f2e45ab9adb0f079c545b8168d8abe617fc2ad31b126231dc92f1"
Sleep 300ms
Enter
Sleep 3s

# Alice: add Bob
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd add bob 62850c7d33603b23f2156f05140ed24d453e6a5cde1a682f5550e9c3bc3b19fb"
Sleep 300ms
Enter
Sleep 3s

# === 3. Direct messages ===

# Alice: DM bob
Type 'talkd dm bob "hey bob, how are you"'
Sleep 300ms
Enter
Sleep 6s

# Bob: read DM
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd read dm:alice"
Sleep 300ms
Enter
Sleep 4s

# Bob: reply
Type 'talkd dm alice "doing great, thanks"'
Sleep 300ms
Enter
Sleep 8s

# Alice: read reply
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd read dm:bob"
Sleep 300ms
Enter
Sleep 4s

# === 4. Channel ===

# Alice: create channel
Type "talkd create team"
Sleep 300ms
Enter
Sleep 5s

# Bob: join channel
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd join f6jnczxorippcbtfkqpjleie7c35pryjdqwexi7bmge7l5cyesgadmcdjxngrdzoiwvzvwypa6ofiw4bndmkxzqx7qvnggysmiy5zexra4ac62duoryhgorpf5sxkyzrfuys44tfnrqxsltogaxgs4tpnawwgylomfzhsltjojxwqltmnfxgwlrpaeab6m4ev3lm4aqbaapthbfovthqgaiaycuact5lwqbqcajkaar4k6vrzaaqaaaaaaaaabrc4p3agaibfiachrl2wheacdmrcsmedkgqjhr7maybaevaai6fpky4qaiqheuoi3i3g6toh5qdar2gkyln"
Sleep 300ms
Enter
Sleep 5s

# Alice: send in channel
Ctrl+B
Type "o"
Sleep 500ms
Type 'talkd send team "hello team, welcome"'
Sleep 300ms
Enter
Sleep 6s

# Bob: read channel
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd read team"
Sleep 300ms
Enter
Sleep 4s

# === 5. Status ===

# Alice: status overview
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd status"
Sleep 300ms
Enter
Sleep 5s

Sleep 3s
