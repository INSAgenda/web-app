When some code is shared between multiple pages, place it into a file in `/common-code`.  
Then, replace it by the following code.  
Make sure to rename `PATH` to the path to the file (without the extension).  

```js
<script common_code="PATH">var d=document;async function u(c,i){var f=d.createElement("div");f.innerHTML=await (await fetch(i)).text();for(var g=0;g<f.childNodes.length;g++){var a=f.childNodes[g];if(1==a.nodeType){var h=d.createElement(a.tagName);h.innerHTML=a.innerHTML;for(var b=0;b<a.attributes.length;b++)h.setAttribute(a.attributes[b].name,a.attributes[b].value);c.parentNode.insertBefore(h,c),a.remove()}}c.remove()}var e=d.currentScript;u(e,"/common-code/"+e.getAttribute("common_code")+".html")</script>
```

The production server will inline the code for optimization purposes.


Note: Here is the original unminified code

```js
var d = document;
async function u(a,b){
    var n = d.createElement("div");
    n.innerHTML=await (await fetch(b)).text();
    for (var i = 0; i < n.childNodes.length; i++) {
        var c = n.childNodes[i];
        if (c.nodeType == 1) {
            var nc = d.createElement(c.tagName);
            nc.innerHTML = c.innerHTML;
            for (var j = 0; j < c.attributes.length; j++) {
                nc.setAttribute(c.attributes[j].name, c.attributes[j].value);
            }
            a.parentNode.insertBefore(nc,a);
            c.remove();
        }
    }
    a.remove();
}
var e=d.currentScript;
u(e,"/common-code/"+e.getAttribute("common_code")+".html")
```
