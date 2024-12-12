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

new-c-system NAME:
  touch src/bin/client/systems/{{NAME}}.rs
  echo -e "use bevy::prelude::*;\n" >> src/bin/client/systems/{{NAME}}.rs
  echo 'pub fn {{NAME}}() {}' >> src/bin/client/systems/{{NAME}}.rs
  $EDITOR src/bin/client/systems/mod.rs src/bin/client/systems/{{NAME}}.rs

new-s-system NAME:

new-system NAME:

