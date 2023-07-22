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
        fetchRandomQuestion();
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

document.addEventListener("DOMContentLoaded", function() {
    document.getElementById("defaultOpen").click();

    document.getElementById("buttonAgain").onclick = function() {submitAnswer(false, 0);};
    document.getElementById("buttonHard").onclick = function() {submitAnswer(true, 1);};
    document.getElementById("buttonGood").onclick = function() {submitAnswer(true, 2);};
    document.getElementById("buttonEasy").onclick = function() {submitAnswer(true, 3);};

    // Fetch the initial question after the page has loaded
    fetchRandomQuestion();
});
