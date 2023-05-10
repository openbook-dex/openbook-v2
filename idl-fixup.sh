#!/bin/bash

# Anchor works purely on a token level and does not know that the index types
# are just type aliases for a primitive type. This hack replaces them with the
# primitive in the idl json and types ts file.
TYPES_TO_REPLACE=(
  "MarketIndex"
  "NodeHandle"
  "usize"
)

pairs=()
for type in ${TYPES_TO_REPLACE[@]}; do
  if [[ "$type" == "usize" ]]; then
    pairs+=("usize u64")
  else
    value=$(grep -r "type $type" "./programs/openbook-v2/src" | awk '{print $NF}' | tr -d ';')
    pairs+=("$type $value")
  fi
done

for pair_str in "${pairs[@]}"; do
  pair=($pair_str)
  perl -0777 -pi -e "s/\{\s*\"defined\":\s*\"${pair[0]}\"\s*\}/\"${pair[1]}\"/g" \
    target/idl/openbook_v2.json target/types/openbook_v2.ts
done

# Anchor puts all enums in the IDL, independent of visibility. And then it
# errors on enums that have tuple variants. This hack drops these from the idl.
perl -0777 -pi -e 's/ *{\s*"name": "NodeRef(?<nested>(?:[^{}[\]]+|\{(?&nested)\}|\[(?&nested)\])*)\},\n//g' \
  target/idl/openbook_v2.json target/types/openbook_v2.ts
