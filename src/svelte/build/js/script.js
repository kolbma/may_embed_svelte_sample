function sample() {
    document.getElementById('sample').textContent = 'Set by JavaScript: Ferris likes Svelte';
}

document.addEventListener('DOMContentLoaded', (event) => {
    sample();
});
