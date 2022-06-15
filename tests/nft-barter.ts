import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { NftBarter } from "../target/types/nft_barter";
import { PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { assert } from "chai";

describe("anchor-escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.NftBarter as Program<NftBarter>;

  let mintA = null;
  let mintB = null;
  let initializerTokenAccountA = null;
  let initializerTokenAccountB = null;
  let takerTokenAccountA = null;
  let takerTokenAccountB = null;
  let vault_account_pda = null;
  let vaultSolAccountPda = null;
  let vault_authority_pda = null;

  const takerAmount = 1_000;
  const initializerAmount = 500;

  const initializerStartSolAmount = 2_000_000_000;
  const takerStartSolAmount = 5_000_000_000;
  const initializerAdditionalSolAmount = 1_000_000_000; // lamport
  const takerAdditionalSolAmount = 3_000_000_000; // lamport

  const escrowAccount = anchor.web3.Keypair.generate();
  const vaultSolAccount = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const initializerMainAccount = anchor.web3.Keypair.generate();
  const takerMainAccount = anchor.web3.Keypair.generate();

  it("Initialize program state", async () => {
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

    mintA = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    mintB = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    initializerTokenAccountA = await mintA.createAccount(
      initializerMainAccount.publicKey
    );
    takerTokenAccountA = await mintA.createAccount(takerMainAccount.publicKey);

    initializerTokenAccountB = await mintB.createAccount(
      initializerMainAccount.publicKey
    );
    takerTokenAccountB = await mintB.createAccount(takerMainAccount.publicKey);

    await mintA.mintTo(
      initializerTokenAccountA,
      mintAuthority.publicKey,
      [mintAuthority],
      initializerAmount
    );

    await mintB.mintTo(
      takerTokenAccountB,
      mintAuthority.publicKey,
      [mintAuthority],
      takerAmount
    );

    let _initializerTokenAccountA = await mintA.getAccountInfo(
      initializerTokenAccountA
    );
    let _takerTokenAccountB = await mintB.getAccountInfo(takerTokenAccountB);

    assert.ok(_initializerTokenAccountA.amount.toNumber() == initializerAmount);
    assert.ok(_takerTokenAccountB.amount.toNumber() == takerAmount);
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
