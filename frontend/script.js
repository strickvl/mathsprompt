function clearForm() {
    document.getElementById('myForm').reset();
}

function submitForm() {
    const questionField = document.getElementById("question");
    const tagsField = document.getElementById("tags");

    const text = questionField.value;
    const tags = tagsField.value;

    fetch('http://localhost:8080/', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ text: text, tags: tags }),
    })
    .then(response => {
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        return response.text();
    })
    .then(data => {
        console.log(data);
        // Clear the question field
        questionField.value = "";
    })
    .catch((error) => {
        console.error('Error:', error);
    });
}

function openTab(evt, tabName) {
    var i, tabcontent, tablinks;

    tabcontent = document.getElementsByClassName("tabcontent");
    for (i = 0; i < tabcontent.length; i++) {
        tabcontent[i].style.display = "none";
    }

    tablinks = document.getElementsByClassName("tablinks");
    for (i = 0; i < tablinks.length; i++) {
        tablinks[i].className = tablinks[i].className.replace(" active", "");
    }

    document.getElementById(tabName).style.display = "block";
    evt.currentTarget.className += " active";

    // Fetch a random question when the Review tab is opened
    // Fetch a random question when the Review tab is opened
    if (tabName === 'Review') {
        console.log('Attempting to fetch a random question...');  // Log for debugging
        fetch('http://localhost:8080/random')
        .then(response => {
            console.log('Received response:', response);  // Log for debugging
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            return response.json();
        })
        .then(data => {
            console.log('Received data:', data);  // Log for debugging
            // Assuming the question text is in the 'text' field of the returned object
            document.getElementById('reviewBox').innerText = data.text;
        })
        .catch((error) => {
            console.error('Error:', error);
        });
    }
}

// Get the element with id="defaultOpen" and click on it
document.getElementById("defaultOpen").click();

window.onload = function() {
    document.getElementById("defaultOpen").click();
}
