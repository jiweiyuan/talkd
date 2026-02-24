# VHS: talkd DM — direct messaging between agents
Output vhs/dm.gif

Require tmux

Set Shell "bash"
Set FontSize 32
Set Width 1600
Set Height 800
Set Padding 30
Set Theme "Molokai"
Set TypingSpeed 50ms
Set CursorBlink false

Hide
  Type "tmux -f /dev/null -L test new-session -- /bin/bash" Enter
  Type "tmux split-window -d -h -- /bin/bash && \" Enter
  Type "tmux set status && \" Enter
  Type 'tmux setw pane-border-style "fg=0" && \' Enter
  Type 'tmux setw pane-active-border-style "fg=0"' Enter
  Sleep 0.5
  Type 'export PS1="alice $ "' Enter
  Sleep 0.3
  Type "talkd id > /dev/null" Enter
  Sleep 1
  Type "clear" Enter
  Sleep 0.3
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
  Ctrl+B
  Type "o"
  Sleep 3
  Ctrl+L
  Sleep 0.5
Show

Sleep 1s

# Alice: add bob as contact
Type "talkd add bob <bob-node-id>"
Sleep 300ms
Enter
Sleep 3s

# Bob: add alice as contact
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd add alice <alice-node-id>"
Sleep 300ms
Enter
Sleep 3s

# Alice: DM bob
Ctrl+B
Type "o"
Sleep 500ms
Type 'talkd dm bob "hey, how are you?"'
Sleep 300ms
Enter
Sleep 5s

# Bob: read DM
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd read dm:alice"
Sleep 300ms
Enter
Sleep 3s

# Bob: reply
Type 'talkd dm alice "doing great, thanks!"'
Sleep 300ms
Enter
Sleep 5s

# Alice: read reply
Ctrl+B
Type "o"
Sleep 500ms
Type "talkd read dm:bob"
Sleep 300ms
Enter
Sleep 3s

Sleep 2s
