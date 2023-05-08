

// async function postSearch(url = '', data = {}) {
    
// }

// const res = await postSearch('/api/search', { answer: 42  });

fetch("/api/search", {
    method: 'POST',
    headers: {
        'Content-Type': 'text/plain',
    },
    body: "haiaaa js, ms css html.  ",

}).then((response) => console.log(response, 'response'))