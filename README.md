
# 🎯 Solana + Anchor training 

## 📌 Overall Learning Goals

✅ Gain a deep understanding of Solana’s smart contract architecture  
✅ Learn to develop, deploy, and interact with on-chain programs using Anchor  
✅ Master account management, token handling, and program security  
✅ Explore advanced concepts like CPI, PDAs, and vesting mechanisms  
✅ Build and deploy a real-world Solana presale contract  

---

## 📚 Lesson 1: Setting Up the Development Environment

### 🎯 Objective:
Set up the necessary tools to develop on Solana using Anchor.

### ✅ Tasks:
- Install Rust and Solana CLI  
- Install Node.js and Yarn  
- Install Anchor framework  
- Set up a local Solana test validator  
- Airdrop some SOL tokens for testing  
- Deploy a basic example program using Anchor  

### 📖 Resources:
- [Solana Docs](https://docs.solana.com)  
- [Anchor Book](https://book.anchor-lang.com)  

---

## 📚 Lesson 2: Understanding Solana and Anchor Basics

### 🎯 Objective:
Learn key Solana concepts and how Anchor simplifies smart contract development.

### ✅ Tasks:
- Understand Solana's architecture (accounts, transactions, programs)  
- Learn about PDAs (Program Derived Addresses)  
- Explore Anchor's program structure (lib.rs, accounts, instructions)  
- Write and deploy a simple counter program  
- Testing and Debugging  

### 🛠 Project:
Build a counter that increments/decrements a value stored in an account.

---

## 📚 Lesson 3: Working with Accounts and Data Storage

### 🎯 Objective:
Manage on-chain data with Solana accounts.

### ✅ Tasks:
- Understand Solana accounts (data storage, ownership, and rent)  
- Learn Anchor’s `#[account]` macro for defining data structures  
- Implement a program that stores reviews (reviewer address, restaurant, review text, and rating)  
- Testing and Debugging  

### 🛠 Project:
Create a program that allows users to create or edit a review for any restaurant.

---

## 📚 Lesson 4: Writing and Executing Instructions

### 🎯 Objective:
Learn how to process user inputs with program instructions.

### ✅ Tasks:
- Define instruction handlers in Anchor  
- Use context constraints to validate accounts  
- Implement custom program errors for better debugging  
- Write a program that allows users to deposit and withdraw SOL  
- Testing and Debugging  

### 🛠 Project:
Implement a simple bank app where users can deposit/withdraw SOL.

---

## 📚 Lesson 5: Cross-Program Invocation (CPI)

### 🎯 Objective:
Learn how to interact with other Solana programs.

### ✅ Tasks:
- Understand CPI (Cross-Program Invocation)  
- Learn how to call System Program for SOL transfers  
- Interact with the Token Program to handle SPL tokens  
- Implement a basic staking contract using CPI  
- Testing and Debugging  

### 🛠 Project:
Build a simple staking contract where users deposit SPL tokens and earn rewards.

---

## 📚 Lesson 6: Build and Deploy a Presale Contract

### 🎯 Objective:
Develop a Solana Presale Smart Contract with vesting mechanics.

### ✅ Tasks:
- Implement an authority role that can create presale events  
- Support presales in SOL or SPL tokens  
- Ensure purchased tokens follow a vesting schedule:  
  - 50% released at TGE (Token Generation Event)  
  - Remaining 50% linearly vested over 2 months  
- Deploy the contract on Solana Devnet  
- Testing and Debugging  

### 🏁 Final Project:
Create a full Presale Program with Vesting Mechanism.

---
