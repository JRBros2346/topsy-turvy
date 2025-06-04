# Documentation for Reverse Code (Backend)

## → User Login:

Input from Users: 

- Name
- Email

So the users can login using Name and Email, and Backend should create a way (like a tracking id) to keep track of that particular user

To prevent the user from Plagiarism if two user login with the same mail id, log out (or give 3 warning) all the currently and trying to login devices

Brute forcing the server to solve a problem will result in a immediate termination

Also make more problem statement to prevent the user from completing all

## → Timer

We need to show the problems to the users for around 1min 

As soon as the user clicks a Problem Statement, we should start a timer and when the user successfully submits a problem we also need to note the timer and calculate the time taken to solve the problem

We should have a timer in frontend that starts when that particular problem is started and should stop when successfully completed that problem or pause the time when user deselects that problem
So it should have a timer in reverse order (overall event timer) and a timer in forward (for each problem to calculate the individual time) in frontend

### → Language Support

Currently: Python, Java, JS, C++, Rust

Need to implement: Go

### → Various Routes

We need to show the actual output from backend to frontend and handle all those execution code

## Problems Faced:

→ Tell a better way to solve Plagiarism 

→ Penalty for Wrong submission

→ How Leader board needs to be calculated

→ We need a set of Questions with Solution and Test-Case with Hidden-Case in all Difficulty Level 
(3 Easy, 2 Medium, 1 Hard)