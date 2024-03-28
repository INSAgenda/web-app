#!/bin/sh

# file=$(find dist -name '*.js')

# Juliette fixes
# to_replace="cachedBigInt64Memory0 = new BigInt64Array();"
# replacement="cachedBigInt64Memory0 = undefined; try { cachedBigInt64Memory0 = new BigInt64Array(); } catch (e) { console.error('BigInt64Array is not supported'); }"
# sed -i -e "s/$to_replace/$replacement/g" $file
# to_replace="cachedBigInt64Memory0 = new BigInt64Array(wasm.memory.buffer);"
# replacement="cachedBigInt64Memory0 = undefined; try { cachedBigInt64Memory0 = new BigInt64Array(wasm.memory.buffer); } catch (e) { console.error('BigInt64Array is not supported'); }"
# sed -i -e "s/$to_replace/$replacement/g" $file

# file="dist/index.html"

# Fix for web-api preprocessing because it fails to handle missing trailing slashes
# to_replace='.css">'
# replacement='.css"\/>'
# sed -i -e "s/$to_replace/$replacement/g" $file

# Add detection of ressource loading errors
# to_replace='<link rel="preload" '
# replacement='<link rel="preload" onerror="on_load_error()" '
# sed -i -e "s/$to_replace/$replacement/g" $file
# to_replace='<link rel="modulepreload" '
# replacement='<link rel="modulepreload" onerror="on_load_error()" '
# sed -i -e "s/$to_replace/$replacement/g" $file


replace() {
    sed -i "s|$1|$2|" "$TRUNK_STAGING_DIR/index.html"
}

comment_out() {
    replace "$1" "//$1"
}

js_file=$(ls $TRUNK_STAGING_DIR | grep 'web-app-[a-zA-Z0-9]*\.js$')
wasm_file=$(ls $TRUNK_STAGING_DIR | grep 'web-app-[a-zA-Z0-9]*_bg\.wasm$')
replace "JS_FILE" $js_file
replace "WASM_FILE" $wasm_file
replace "<script type=\"module\">" "<script>"
comment_out "import init from"
comment_out "import initializer from"
comment_out "await __trunkInitializer"

