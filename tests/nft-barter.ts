import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { NftBarter } from "../target/types/nft_barter";
import { PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  setAuthority,
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("anchor-escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.NftBarter as Program<NftBarter>;

  let mintA = null;
  let mintB = null;
  let mintC = null;
  let mintD = null;
  let mintE = null;

  let initializerTokenAccountA = null;
  let initializerTokenAccountB = null;
  let initializerTokenAccountC = null;
  let initializerTokenAccountD = null;
  let initializerTokenAccountE = null;

  let takerTokenAccountA = null;
  let takerTokenAccountB = null;
  let takerTokenAccountC = null;
  let takerTokenAccountD = null;
  let takerTokenAccountE = null;

  let vault_account_pda = null;
  let vaultSolAccountPda = null;
  let vault_authority_pda = null;

  const initializerStartSolAmount = 2_000_000_000;
  const takerStartSolAmount = 5_000_000_000;
  const initializerAdditionalSolAmount = 1_000_000_000; // lamport
  const takerAdditionalSolAmount = 3_000_000_000; // lamport

  const escrowAccount = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const initializerMainAccount = anchor.web3.Keypair.generate();
  const takerMainAccount = anchor.web3.Keypair.generate();

  it("Initialize program state", async () => {
    console.log("start airdrop");

    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10_000_000_000),
      "confirmed"
    );

    // Fund Main Accounts
    await provider.send(
      (() => {
        const tx = new Transaction();
        tx.add(
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: initializerMainAccount.publicKey,
            lamports: initializerStartSolAmount,
          }),
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: takerMainAccount.publicKey,
            lamports: takerStartSolAmount,
          })
        );
        return tx;
      })(),
      [payer]
    );

    console.log("start createMint");

    mintA = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    mintB = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    mintC = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    mintD = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    mintE = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    console.log("start getOrCreateAssociatedTokenAccount");

    // mintA mintBはinitializerが保有している
    initializerTokenAccountA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      initializerMainAccount,
      mintA,
      initializerMainAccount.publicKey
    );
    initializerTokenAccountB = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      initializerMainAccount,
      mintB,
      initializerMainAccount.publicKey
    );

    // mintC mintD mintEはtakerが保有している
    takerTokenAccountC = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      takerMainAccount,
      mintC,
      takerMainAccount.publicKey
    );
    takerTokenAccountD = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      takerMainAccount,
      mintD,
      takerMainAccount.publicKey
    );
    takerTokenAccountE = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      takerMainAccount,
      mintE,
      takerMainAccount.publicKey
    );

    // 持っていないNFTのtoken accountはrust側で確認して作る

    console.log("start mintTo");

    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintA, // Mint for the account
      initializerTokenAccountA.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );
    console.log("start mintTo1");
    await setAuthority(
      provider.connection,
      payer, // Payer of the transaction fees
      mintA, // Account
      mintAuthority, // Current authority
      0, // Authority type: "0" represents Mint Tokens
      null // Setting the new Authority to null
    );
    console.log("start mintTo2");
    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintB, // Mint for the account
      initializerTokenAccountB.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );
    await setAuthority(
      provider.connection,
      payer, // Payer of the transaction fees
      mintB, // Account
      mintAuthority, // Current authority
      0, // Authority type: "0" represents Mint Tokens
      null // Setting the new Authority to null
    );

    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintC, // Mint for the account
      takerTokenAccountC.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );
    await setAuthority(
      provider.connection,
      payer, // Payer of the transaction fees
      mintC, // Account
      mintAuthority, // Current authority
      0, // Authority type: "0" represents Mint Tokens
      null // Setting the new Authority to null
    );

    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintD, // Mint for the account
      takerTokenAccountD.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );
    await setAuthority(
      provider.connection,
      payer, // Payer of the transaction fees
      mintD, // Account
      mintAuthority, // Current authority
      0, // Authority type: "0" represents Mint Tokens
      null // Setting the new Authority to null
    );

    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintE, // Mint for the account
      takerTokenAccountE.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );
    await setAuthority(
      provider.connection,
      payer, // Payer of the transaction fees
      mintE, // Account
      mintAuthority, // Current authority
      0, // Authority type: "0" represents Mint Tokens
      null // Setting the new Authority to null
    );

    console.log("start assertion");

    let _initializerTokenAccountA = await mintA.getAccountInfo(
      initializerTokenAccountA
    );
    assert.ok(_initializerTokenAccountA.amount.toNumber() == 1);

    let _initializerTokenAccountB = await mintB.getAccountInfo(
      initializerTokenAccountB
    );
    assert.ok(_initializerTokenAccountB.amount.toNumber() == 1);

    let _takerTokenAccountC = await mintC.getAccountInfo(takerTokenAccountC);
    assert.ok(_takerTokenAccountC.amount.toNumber() == 1);

    let _takerTokenAccountD = await mintC.getAccountInfo(takerTokenAccountD);
    assert.ok(_takerTokenAccountD.amount.toNumber() == 1);

    let _takerTokenAccountE = await mintC.getAccountInfo(takerTokenAccountE);
    assert.ok(_takerTokenAccountE.amount.toNumber() == 1);
  });

  it("Initialize escrow", async () => {
    const [_vault_account_pda, _vault_account_bump] =
      await PublicKey.findProgramAddress(
        [Buffer.from(anchor.utils.bytes.utf8.encode("token-seed"))],
        program.programId
      );
    vault_account_pda = _vault_account_pda;

    const [_vaultSolAccountPda, _vaultSolAccountBump] =
      await PublicKey.findProgramAddress(
        [Buffer.from(anchor.utils.bytes.utf8.encode("vault-sol-account"))],
        program.programId
      );
    vaultSolAccountPda = _vaultSolAccountPda;

    const [_vault_authority_pda, _vault_authority_bump] =
      await PublicKey.findProgramAddress(
        [Buffer.from(anchor.utils.bytes.utf8.encode("escrow"))],
        program.programId
      );
    vault_authority_pda = _vault_authority_pda;

    await program.rpc.initialize(
      new anchor.BN(initializerAmount),
      new anchor.BN(initializerAdditionalSolAmount),
      new anchor.BN(takerAmount),
      new anchor.BN(takerAdditionalSolAmount),
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          taker: takerMainAccount.publicKey,
          mint: mintA.publicKey,
          vaultAccount: vault_account_pda,
          initializerDepositTokenAccount: initializerTokenAccountA,
          initializerReceiveTokenAccount: initializerTokenAccountB,
          vaultSolAccount: vaultSolAccountPda,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
          await program.account.escrowAccount.createInstruction(escrowAccount),
        ],
        signers: [escrowAccount, initializerMainAccount], // escrowAccount抜かすとエラーになる
      }
    );

    let _vault = await mintA.getAccountInfo(vault_account_pda);
    console.log("_vault.owner", _vault.owner);
    console.log("vault_authority_pda", vault_authority_pda);

    let _escrowAccount = await program.account.escrowAccount.fetch(
      escrowAccount.publicKey
    );

    // Check that the new owner is the PDA.
    assert.ok(_vault.owner.equals(vault_authority_pda));

    // Check that the values in the escrow account match what we expect.
    assert.ok(
      _escrowAccount.initializerKey.equals(initializerMainAccount.publicKey)
    );
    assert.ok(_escrowAccount.initializerAmount.toNumber() == initializerAmount);
    assert.ok(_escrowAccount.takerAmount.toNumber() == takerAmount);
    assert.ok(
      _escrowAccount.initializerDepositTokenAccount.equals(
        initializerTokenAccountA
      )
    );
    assert.ok(
      _escrowAccount.initializerReceiveTokenAccount.equals(
        initializerTokenAccountB
      )
    );
  });

  it("Exchange escrow state", async () => {
    await program.rpc.exchange({
      accounts: {
        taker: takerMainAccount.publicKey,
        takerDepositTokenAccount: takerTokenAccountB,
        takerReceiveTokenAccount: takerTokenAccountA,
        initializerDepositTokenAccount: initializerTokenAccountA,
        initializerReceiveTokenAccount: initializerTokenAccountB,
        vaultSolAccount: vaultSolAccountPda, // ここの値が違うと Error: 3012: The program expected this account to be already initialized
        initializer: initializerMainAccount.publicKey,
        escrowAccount: escrowAccount.publicKey,
        vaultAccount: vault_account_pda,
        vaultAuthority: vault_authority_pda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [takerMainAccount],
    });

    let _takerTokenAccountA = await mintA.getAccountInfo(takerTokenAccountA);
    let _takerTokenAccountB = await mintB.getAccountInfo(takerTokenAccountB);
    let _initializerTokenAccountA = await mintA.getAccountInfo(
      initializerTokenAccountA
    );
    let _initializerTokenAccountB = await mintB.getAccountInfo(
      initializerTokenAccountB
    );

    assert.ok(_takerTokenAccountA.amount.toNumber() == initializerAmount);
    assert.ok(_initializerTokenAccountA.amount.toNumber() == 0);
    assert.ok(_initializerTokenAccountB.amount.toNumber() == takerAmount);
    assert.ok(_takerTokenAccountB.amount.toNumber() == 0);

    // initializerで署名してもvaultからsol引き落とそうとしたらError: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account
    // のエラーがでるが、テストで保証しておきたい

    // 追加SOLのチェック token closeしないとずれる
    // close = initializerありなしでもずれる
    const initializer = await provider.connection.getAccountInfo(
      initializerMainAccount.publicKey
    );
    console.log("initializer.lamports", initializer.lamports);
    console.log("initializerStartSolAmount", initializerStartSolAmount);
    console.log(
      "initializerAdditionalSolAmount",
      initializerAdditionalSolAmount
    );
    /*
    assert.ok(
      _escrowAccount.initializerAdditionalSolAmount.toNumber() ==
        initializerAdditionalSolAmount
    );
    assert.ok(
      _escrowAccount.takerAdditionalSolAmount.toNumber() ==
        takerAdditionalSolAmount
    );
    const initializer = await provider.connection.getAccountInfo(
      initializerMainAccount.publicKey
    );
    console.log("initializer.lamports", initializer.lamports);
    console.log("initializerStartSolAmount", initializerStartSolAmount);
    console.log(
      "initializerAdditionalSolAmount",
      initializerAdditionalSolAmount
    );
    assert.ok(
      initializer.lamports ===
        initializerStartSolAmount - initializerAdditionalSolAmount
    );
    const taker = await provider.connection.getAccountInfo(
      takerMainAccount.publicKey
    );
    assert.ok(
      taker.lamports === takerStartSolAmount - takerAdditionalSolAmount
    );
    */
  });

  /*
  it("Initialize escrow and cancel escrow by A", async () => {
    // Put back tokens into initializer token A account.
    await mintA.mintTo(
      initializerTokenAccountA,
      mintAuthority.publicKey,
      [mintAuthority],
      initializerAmount
    );

    await program.rpc.initialize(
      new anchor.BN(initializerAmount),
      new anchor.BN(initializer_additional_sol_amount),
      new anchor.BN(takerAmount),
      new anchor.BN(taker_additional_sol_amount),
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          taker: takerMainAccount.publicKey,
          mint: mintA.publicKey,
          vaultAccount: vault_account_pda,
          initializerDepositTokenAccount: initializerTokenAccountA,
          initializerReceiveTokenAccount: initializerTokenAccountB,
          vaultSolAccount: vaultSolAccountPda,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
          await program.account.escrowAccount.createInstruction(escrowAccount),
        ],
        signers: [escrowAccount, initializerMainAccount], // escrowAccount抜かすとエラーになる
      }
    );

    // Cancel the escrow.
    await program.rpc.cancelByInitializer({
      accounts: {
        initializer: initializerMainAccount.publicKey,
        vaultAccount: vault_account_pda,
        vaultAuthority: vault_authority_pda,
        initializerDepositTokenAccount: initializerTokenAccountA,
        vaultSolAccount: vaultSolAccountPda,
        escrowAccount: escrowAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [initializerMainAccount],
    });

    // Check the final owner should be the provider public key.
    const _initializerTokenAccountA = await mintA.getAccountInfo(
      initializerTokenAccountA
    );
    assert.ok(
      _initializerTokenAccountA.owner.equals(initializerMainAccount.publicKey)
    );

    // Check all the funds are still there.
    assert.ok(_initializerTokenAccountA.amount.toNumber() == initializerAmount);

    // token AccountBのチェックを入れてみた
    const _initializerTokenAccountB = await mintB.getAccountInfo(
      initializerTokenAccountB
    );
    assert.ok(_initializerTokenAccountB.amount.toNumber() == takerAmount);
  });

  it("Initialize escrow and cancel escrow by B", async () => {
    await program.rpc.initialize(
      new anchor.BN(initializerAmount),
      new anchor.BN(initializer_additional_sol_amount),
      new anchor.BN(takerAmount),
      new anchor.BN(taker_additional_sol_amount),
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          taker: takerMainAccount.publicKey,
          mint: mintA.publicKey,
          vaultAccount: vault_account_pda,
          initializerDepositTokenAccount: initializerTokenAccountA,
          initializerReceiveTokenAccount: initializerTokenAccountB,
          vaultSolAccount: vaultSolAccount.publicKey,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
          await program.account.escrowAccount.createInstruction(escrowAccount),
        ],
        signers: [escrowAccount, initializerMainAccount], // escrowAccount抜かすとエラーになる
      }
    );

    // Cancel the escrow.
    await program.rpc.cancelByTaker({
      accounts: {
        initializer: initializerMainAccount.publicKey,
        taker: takerMainAccount.publicKey,
        vaultAccount: vault_account_pda,
        vaultAuthority: vault_authority_pda,
        initializerDepositTokenAccount: initializerTokenAccountA,
        vaultSolAccount: vaultSolAccount.publicKey,
        escrowAccount: escrowAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [takerMainAccount],
    });

    // Check the final owner should be the provider public key.
    const _initializerTokenAccountA = await mintA.getAccountInfo(
      initializerTokenAccountA
    );
    assert.ok(
      _initializerTokenAccountA.owner.equals(initializerMainAccount.publicKey)
    );

    // Check all the funds are still there.
    assert.ok(_initializerTokenAccountA.amount.toNumber() == initializerAmount);

    // token AccountBのチェックを入れてみた
    const _initializerTokenAccountB = await mintB.getAccountInfo(
      initializerTokenAccountB
    );
    assert.ok(_initializerTokenAccountB.amount.toNumber() == takerAmount);
  });
*/
  // initializerがSOL払う場合

  // takerがSOL払う場合
});
