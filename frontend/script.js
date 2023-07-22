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

function submitAnswer(answeredCorrectly, ease) {
    fetch('http://localhost:8080/submit_answer', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ question_id: currentQuestionId, answered_correctly: answeredCorrectly, ease: ease }),
    })
    .then(response => response.json())
    .then(data => {
        console.log(data);
        // Fetch a new question after submitting the answer
        if (document.getElementById('Review').style.display === 'block') {
            fetchRandomQuestion();
        } else if (document.getElementById('Focus').style.display === 'block') {
            fetchRandomQuestionByTag();
        }
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
    if (tabName === 'Review') {
        fetchRandomQuestion();
    }
    if (tabName === 'Focus') {
        fetchTags();
    }
}

function fetchRandomQuestion() {
    fetch('http://localhost:8080/random')
    .then(response => response.json())
    .then(data => {
        // Store the question ID and display the question text
        currentQuestionId = data.id;
        document.getElementById('reviewBox').innerText = data.text;
    })
    .catch((error) => {
        console.error('Error:', error);
    });
}

function fetchTags() {
    fetch('http://localhost:8080/tags')
        .then(response => response.json())
        .then(data => {
            // Sort the tags alphabetically
            data.sort();

            let dropdown = document.getElementById('tagSelect');
            dropdown.length = 0;

            // Add a blank option at the top
            let defaultOption = document.createElement('option');
            defaultOption.text = '';
            dropdown.add(defaultOption);

            data.forEach((tag) => {
                let option = document.createElement('option');
                option.text = tag;
                option.value = tag;
                dropdown.add(option);
            });
        })
        .catch((error) => {
            console.error('Error:', error);
        });
}



function fetchRandomQuestionByTag() {
    let tag = document.getElementById('tagSelect').value;
    fetch(`http://localhost:8080/random?tag=${tag}`)
        .then(response => response.json())
        .then(data => {
            // Store the question ID and display the question text
            currentQuestionId = data.id;
            document.getElementById('focusBox').innerText = data.text;
        })
        .catch((error) => {
            console.error('Error:', error);
        });
}

document.addEventListener("DOMContentLoaded", function() {

    document.getElementById("buttonAgain").onclick = function() {submitAnswer(false, 0);};
    document.getElementById("buttonHard").onclick = function() {submitAnswer(true, 1);};
    document.getElementById("buttonGood").onclick = function() {submitAnswer(true, 2);};
    document.getElementById("buttonEasy").onclick = function() {submitAnswer(true, 3);};

    // ... Add event listeners for the new review buttons in the Focus tab ...
    document.getElementById("focusButtonAgain").onclick = function() {submitAnswer(false, 0);};
    document.getElementById("focusButtonHard").onclick = function() {submitAnswer(true, 1);};
    document.getElementById("focusButtonGood").onclick = function() {submitAnswer(true, 2);};
    document.getElementById("focusButtonEasy").onclick = function() {submitAnswer(true, 3);};

    // Fetch the initial question after the page has loaded
    document.getElementById("defaultOpen").click();
    fetchRandomQuestion();
});

