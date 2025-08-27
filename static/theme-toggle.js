const storageKey = 'setting-theme';
const authoThemeKey = 'auto-theme';

const getSystemPreference = () => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

const getColorPreference = () => {
    if (localStorage.getItem(storageKey))
        return localStorage.getItem(storageKey);
    else
        return getSystemPreference();
}

document.reflectTheme = reflectPreference = function () {
    let theme = document.firstElementChild.getAttribute('data-theme');
    if (theme) return;

    if (localStorage.getItem(authoThemeKey) === 'true') {
        document.firstElementChild.setAttribute('data-theme', getSystemPreference());
    } else {
        document.firstElementChild.setAttribute('data-theme', getColorPreference())
    }
}

document.reflectTheme()
window.onload = () => {
    document.reflectTheme()
}

window
    .matchMedia('(prefers-color-scheme: dark)')
    .addEventListener('change', ({matches:isDark}) => {     
        // TODO: same call in webapp when selecting auto-theme
        reflectPreference()
    })
