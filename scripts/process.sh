#!/usr/bin/env bash

set -e

: "${ROLLUP_MESSAGE_INDEX:=/var/lib/tezos-place/next_index}"
: "${ROLLUP_EXTERNAL_MESSAGE_LOG:=/var/lib/tezos-place/external_message_log}"
: "${PASSWORD:=./secret/password}"

increment() {
  local file_path="$1"

  # Check if the file exists
  if [[ -f "$file_path" ]]; then
    # Read the number from the file
    local number
    number=$(<"$file_path")
  else
    # Set the initial number if the file doesn't exist
    local number=0
  fi

  # Increment the number
  ((number++))

  # Store the updated number back into the file
  echo "$number" >"$file_path"
}

# Check if the file exists
if [[ -f "$ROLLUP_MESSAGE_INDEX" ]]; then
  # Read the number from the file
  index=$(<"$ROLLUP_MESSAGE_INDEX")
else
  echo "1" > "$ROLLUP_MESSAGE_INDEX"
  # Set the initial number if the file doesn't exist
  index=1
fi

set -x
next_message=$(sed -n "${index}p" "$ROLLUP_EXTERNAL_MESSAGE_LOG")
echo "bar"

if [[ -n "${next_message}" ]]; then
  # Perform actions when the line is not empty (i.e., has a non-zero length)
  echo "Processing message $index: $next_message"
  hex_message=$(echo "$next_message" | xxd -p -c 1000000000)
  hex="74$hex_message"
  octez-client --wait 1 -f "$PASSWORD" send smart rollup message "hex:[\"$hex\"]" from prod
  echo "Done, incrementing message index"
  increment "$ROLLUP_MESSAGE_INDEX"
else
  echo "No new messages to process"
fi
