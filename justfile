default:
  just -l

new-window NAME CMD:
  tmux new-w -t rt-chess -n "{{NAME}}"
  tmux send-keys -t rt-chess:"{{NAME}}" "{{CMD}}" ENTER

tmux:
  tmux new -ds rt-chess -n "README"
  tmux send-keys -t rt-chess:README 'nv ./README.md "+set wrap"' ENTER
  # @just new-window "GUI" "nv ./gui/MIDI-Tracker.pygame +'setfiletype python'"
  @just new-window "Edit" ""
  @just new-window "Run" ""
  @just new-window "Git" "git status"
  @just new-window "Misc" ""
  tmux a -t rt-chess
