#!/usr/bin/env bash

presence=$(myarchy-battery presence)
state=$(myarchy-battery state)

if [[ $presence == true ]]; then
  if [[ $state == "charging" ]]; then
    echo -n "(+) "
  fi
  echo $(myarchy-battery percentage)"%"
  if [[ $state == "discharging" ]]; then
    echo -n " remaining"
  fi
fi

echo ''
