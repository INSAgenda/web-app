// Animate radio inputs
var radios = document.getElementsByClassName('fancy-radio-input');
for (const radio of radios) {
    radio.addEventListener('click', function () {
        for (const radio of radios) {
            if (radio.checked == false) {
                radio.parentElement.classList.remove('fancy-radio-label-active');
            }
        }
        var parent = this.parentNode;
        parent.classList.add('fancy-radio-label-active');
    });
}
