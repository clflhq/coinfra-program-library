import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { NftBarter } from "../target/types/nft_barter";
import { PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import {
  AccountLayout,
  TOKEN_PROGRAM_ID,
  setAuthority,
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
  getAccount,
  createInitializeAccountInstruction,
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

  let vaultAccountPda: anchor.web3.PublicKey = null; // テスト用　使っていない
  let vaultAccountBump: number = null;

  let vaultAccountPdaA: anchor.web3.PublicKey = null; // initializerがmintAを預ける用のvault vaultはPDAにする必要がない
  let vaultAccountBumpA: number = null;
  let vaultAccountA: anchor.web3.Keypair = anchor.web3.Keypair.generate();

  let vaultAccountPdaB: anchor.web3.PublicKey = null; // initializerがmintBを預ける用のvault
  let vaultAccountBumpB: number = null;
  let vaultAccountB: anchor.web3.Keypair = anchor.web3.Keypair.generate();

  let vaultSolAccountPda: anchor.web3.PublicKey = null;
  let vaultSolAccountBump: number = null;

  let vaultAuthorityPda: anchor.web3.PublicKey = null;
  let vaultAuthorityBump: number = null;

  const initializerStartSolAmount = 2_000_000_000;
  const takerStartSolAmount = 5_000_000_000;
  const initializerAdditionalSolAmount = 1_000_000_000; // lamport
  const takerAdditionalSolAmount = 3_000_000_000; // lamport

  const escrowAccount: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  const payer: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  const mintAuthority: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  const initializerMainAccount: anchor.web3.Keypair =
    anchor.web3.Keypair.generate();
  const takerMainAccount: anchor.web3.Keypair = anchor.web3.Keypair.generate();

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
    await setAuthority(
      provider.connection,
      payer, // Payer of the transaction fees
      mintA, // Account
      mintAuthority, // Current authority
      0, // Authority type: "0" represents Mint Tokens
      null // Setting the new Authority to null
    );

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
    console.log(initializerTokenAccountA);
    console.log(initializerTokenAccountA.amount);

    // 以下でNFT数を検証したほうがよさそう？
    // const tokens = await connection.getParsedTokenAccountsByOwner(owner, {
    //  programId: TOKEN_PROGRAM_ID,
    // });
    let _initializerTokenAccountA = await getAccount(
      provider.connection,
      initializerTokenAccountA.address
    );

    console.log(_initializerTokenAccountA);
    console.log(Number(_initializerTokenAccountA.amount));
    assert.ok(Number(_initializerTokenAccountA.amount) == 1);

    let _initializerTokenAccountB = await getAccount(
      provider.connection,
      initializerTokenAccountB.address
    );
    assert.ok(Number(_initializerTokenAccountB.amount) == 1);

    let _takerTokenAccountC = await getAccount(
      provider.connection,
      takerTokenAccountC.address
    );
    assert.ok(Number(_takerTokenAccountC.amount) == 1);

    let _takerTokenAccountD = await getAccount(
      provider.connection,
      takerTokenAccountD.address
    );
    assert.ok(Number(_takerTokenAccountD.amount) == 1);

    let _takerTokenAccountE = await getAccount(
      provider.connection,
      takerTokenAccountE.address
    );
    assert.ok(Number(_takerTokenAccountE.amount) == 1);

    // へんなaddressをtoken account addressでわたしたときの異常系チェック
  });

  it("Initialize escrow", async () => {
    console.log("start creating PDAs");

    const [_vaultAccountPda, _vaultAccountBump] =
      await PublicKey.findProgramAddress(
        [Buffer.from(anchor.utils.bytes.utf8.encode("vault-account-test"))],
        program.programId
      );
    vaultAccountPda = _vaultAccountPda;
    vaultAccountBump = _vaultAccountBump;

    /*
    const [_vaultAccountPdaA, _vaultAccountBumpA] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountA.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaA = _vaultAccountPdaA;
    vaultAccountBumpA = _vaultAccountBumpA;
    console.log("initializerTokenAccountA", initializerTokenAccountA);
    console.log("vaultAccountPdaA", vaultAccountPdaA);
    console.log("vaultAccountBumpA", vaultAccountBumpA);
*/
    const createTempTokenAccountAIx = SystemProgram.createAccount({
      programId: TOKEN_PROGRAM_ID,
      space: AccountLayout.span,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(
        AccountLayout.span
      ),
      fromPubkey: initializerMainAccount.publicKey,
      newAccountPubkey: vaultAccountA.publicKey,
    });
    const initTempAccountAIx = createInitializeAccountInstruction(
      vaultAccountA.publicKey,
      mintA,
      initializerMainAccount.publicKey,
      TOKEN_PROGRAM_ID
    );

    /*
    await setAuthority(
      provider.connection,
      payer, // Payer of the transaction fees
      vaultAccountPdaA, // Account
      anchor.web3.SystemProgram.programId, // Current authority
      2, // Authority type: "2" represents Account Owner
      initializerMainAccount.publicKey // Setting the new Authority to null
    );
    */

    // 以下だとデータ取れない
    // let _vault = await provider.connection.getAccountInfo(vaultAccountPdaA);
    // console.log("_vault", _vault);

    /*
    const [_vaultAccountPdaB, _vaultAccountBumpB] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountB.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaB = _vaultAccountPdaB;
    vaultAccountBumpB = _vaultAccountBumpB;
    console.log("initializerTokenAccountB", initializerTokenAccountB);
    console.log("vaultAccountPdaB", vaultAccountPdaB);
    console.log("vaultAccountBumpB", vaultAccountBumpB);
*/
    const createTempTokenAccountBIx = SystemProgram.createAccount({
      programId: TOKEN_PROGRAM_ID,
      space: AccountLayout.span,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(
        AccountLayout.span
      ),
      fromPubkey: initializerMainAccount.publicKey,
      newAccountPubkey: vaultAccountB.publicKey,
    });
    const initTempAccountBIx = createInitializeAccountInstruction(
      vaultAccountB.publicKey,
      mintB,
      initializerMainAccount.publicKey,
      TOKEN_PROGRAM_ID
    );

    /*
    await setAuthority(
      provider.connection,
      payer, // Payer of the transaction fees
      vaultAccountPdaB, // Account
      anchor.web3.SystemProgram.programId, // Current authority
      2, // Authority type: "2" represents Account Owner
      initializerMainAccount.publicKey // Setting the new Authority to null
    );
    */

    const vaultAccountBumps: number[] = [vaultAccountBumpA, vaultAccountBumpB];
    console.log("vaultAccountBumps", vaultAccountBumps);

    const [_vaultSolAccountPda, _vaultSolAccountBump] =
      await PublicKey.findProgramAddress(
        [
          anchor.utils.bytes.utf8.encode("vault-sol-account"),
          initializerMainAccount.publicKey.toBuffer(),
          takerMainAccount.publicKey.toBuffer(),
        ], // sampleコードではBufferが書いてあったがBufferはいらないと思われる anchor bookにはない
        program.programId
      );
    vaultSolAccountPda = _vaultSolAccountPda;
    vaultSolAccountBump = _vaultSolAccountBump;
    console.log("vaultSolAccountPda", vaultSolAccountPda); // 毎回まったく同じPDAが生成される　programID変えても同じ
    console.log("vaultSolAccountBump", vaultSolAccountBump); // 毎回まったく同じPDAが生成される　programID変えても同じ

    const [_vaultAuthorityPda, _vaultAuthorityBump] =
      await PublicKey.findProgramAddress(
        [
          anchor.utils.bytes.utf8.encode("vault-authority"),
          initializerMainAccount.publicKey.toBuffer(),
          takerMainAccount.publicKey.toBuffer(),
        ], // sampleコードではBufferが書いてあったがBufferはいらないと思われる anchor bookにはない
        program.programId
      );
    vaultAuthorityPda = _vaultAuthorityPda;
    vaultAuthorityBump = _vaultAuthorityBump;
    console.log("vaultAuthorityPda", vaultAuthorityPda);
    console.log("vaultAuthorityBump", vaultAuthorityBump);

    console.log("start initialize");
    console.log(
      "initializerTokenAccountA.address",
      initializerTokenAccountA.address
    );

    // initializerはtoken accountとbump takerは直接initializerに払い出すのでtoken accountのみ
    // writebleである必要があるかどうかは不明？
    const remainingAccounts = [];
    remainingAccounts.push({
      pubkey: initializerTokenAccountA.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountA.publicKey,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountB.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountB.publicKey,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountC.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountD.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountE.address,
      isWritable: true,
      isSigner: false,
    });

    await program.rpc.initialize(
      new anchor.BN(initializerAdditionalSolAmount),
      new anchor.BN(takerAdditionalSolAmount),
      2,
      3,
      Buffer.from(vaultAccountBumps), // Buffer.fromしないとTypeError: Blob.encode[data] requires (length 2) Buffer as src
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          taker: takerMainAccount.publicKey,
          // vaultAccount: vaultAccountPda,
          vaultSolAccount: vaultSolAccountPda,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
          await program.account.escrowAccount.createInstruction(escrowAccount), // 抜かすとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: Program failed to complete
          createTempTokenAccountAIx,
          initTempAccountAIx,
          createTempTokenAccountBIx,
          initTempAccountBIx,
        ],
        remainingAccounts: remainingAccounts,
        signers: [
          escrowAccount,
          initializerMainAccount,
          vaultAccountA,
          vaultAccountB,
        ], // escrowAccount抜かすとError: Signature verification failed
      }
    );
    /*
    console.log("start assertion");
    let _vault = await provider.connection.getAccountInfo(vault_account_pda);
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
    );*/

    /* NFTの数をチェック
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
    */

    // 連続してinitializeしたときのテスト

    // 同じ組み合わせのinitializer, takerで二重に取引できない
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
        vaultAuthority: vaultAuthorityPda,
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
