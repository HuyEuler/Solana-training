import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ReviewProgram } from "../target/types/review_program";
import { assert } from "chai";
import { readFileSync } from "fs";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";

describe("Review Program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.ReviewProgram as Program<ReviewProgram>;

  const user1 = loadLocalSigner("id.json");
  const user2 = loadLocalSigner("wallet-tmp.json");

  it("Submits a review of user 1", async () => {
    const [reviewPDA, bump] = await PublicKey.findProgramAddressSync(
      [Buffer.from("review"), user1.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .submitReview("Pasta La Vista", "Great pasta", 4)
      .accounts({
        review: reviewPDA,
        reviewer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    const storedReview = await program.account.review.fetch(reviewPDA);
    console.log("Stored Review:", storedReview);
    
  });

  it("Submits a review from user 2", async () => {
    const [reviewPDA, bump] = await PublicKey.findProgramAddressSync(
        [Buffer.from("review"), user2.publicKey.toBuffer()],
        program.programId
    );

    await program.methods
      .submitReview("Mikado Sushi Vincom Bà Triệu", "I love sushi here", 5)
      .accounts({
        review: reviewPDA,
        reviewer: user2.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user2])
      .rpc();

    const storedReview = await program.account.review.fetch(reviewPDA);
    console.log("Stored Review:", storedReview);
  });

  it("Edit information", async () => {
    const review = '5ZkcdxAkJ8h37WWybj1S6hZcAwBNV9heEDBK7AVYNee9';
    const storedReview = await program.account.review.fetch(review);
    console.log(storedReview);
    await program.methods
      .editReview("Great food", 4, "Ha Noi")
      .accounts({
        review: review,
        reviewer: user2.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user2])
      .rpc();
      const storedNewReview = await program.account.review.fetch(review);
      console.log(storedNewReview);
  });

  it("Fetches all reviews", async () => {
    const allReviews = await program.account.review.all();
    console.log("All Reviews:", allReviews);

    allReviews.forEach(review => {
      console.log(`Restaurant: ${review.account.restaurant}`);
      console.log(`Review: ${review.account.review}`);
      console.log(`Rating: ${review.account.rating}`);
      console.log(`Reviewer: ${review.account.reviewer.toBase58()}`);
      console.log("----");
    });

    assert.isArray(allReviews);
  });
  
});

interface Signer {
  publicKey: PublicKey;
  secretKey: Uint8Array;
}

function loadLocalSigner(path: String): Signer {
  const keypairPath = `${process.env.HOME}/.config/solana/${path}`;
  const secretKeyString = readFileSync(keypairPath, { encoding: "utf-8" });
  const secretKeyArray = JSON.parse(secretKeyString) as number[];
  const keypair = Keypair.fromSecretKey(new Uint8Array(secretKeyArray));

  return {
      publicKey: keypair.publicKey,
      secretKey: keypair.secretKey,
  };
}
