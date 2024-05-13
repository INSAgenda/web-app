#!/bin/sh

replace() {
    sed -i "s|$1|$2|" "$TRUNK_STAGING_DIR/index.html"
}

comment_out() {
    replace "$1" "//$1"
}

js_file=$(ls $TRUNK_STAGING_DIR | grep 'web-app-[a-zA-Z0-9]*\.js$')
wasm_file=$(ls $TRUNK_STAGING_DIR | grep 'web-app-[a-zA-Z0-9]*_bg\.wasm$')

# Juliette fixes
to_replace="cachedBigInt64Memory0 = new BigInt64Array();"
replacement="cachedBigInt64Memory0 = undefined; try { cachedBigInt64Memory0 = new BigInt64Array(); } catch (e) { console.error('BigInt64Array is not supported'); }"
sed -i -e "s/$to_replace/$replacement/g" $js_file
to_replace="cachedBigInt64Memory0 = new BigInt64Array(wasm.memory.buffer);"
replacement="cachedBigInt64Memory0 = undefined; try { cachedBigInt64Memory0 = new BigInt64Array(wasm.memory.buffer); } catch (e) { console.error('BigInt64Array is not supported'); }"
sed -i -e "s/$to_replace/$replacement/g" $js_file

replace "JS_FILE" $js_file
replace "WASM_FILE" $wasm_file
replace "<script type=\"module\">" "<script>"
replace "Content-Length" "X-Content-Length"
comment_out "import init from"
comment_out "import initializer from"
comment_out "await __trunkInitializer"
