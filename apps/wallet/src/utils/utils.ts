import { ethers, Wallet } from "ethers";

export function generateWallet(): Wallet {
  const wallet = Wallet.createRandom();
  return wallet;
}

export function storePrivateKey(wallet: Wallet): void {
  localStorage.setItem("privateKey", wallet.privateKey);
}

export async function createAndSignTransaction(
  senderWallet: Wallet,
  transaction: any,
): Promise<string> {
  const tx = await senderWallet.signTransaction(transaction);
  return tx;
}

export async function sendTransaction(
  signedTransaction: string,
): Promise<ethers.providers.TransactionResponse> {
  const provider = new ethers.providers.JsonRpcProvider(process.env.REACT_APP_API_URL);
  const txResponse = await provider.sendTransaction(signedTransaction);
  await txResponse.wait();
  return txResponse;
}

export async function getBalance(address: string): Promise<string> {
  if (!process.env.REACT_APP_API_URL) {
    console.error(
      "Spire gateway RPC undefined - returning default balance of 0",
    );
    return Promise.resolve("0");
  }
  const provider = new ethers.providers.JsonRpcProvider(process.env.REACT_APP_API_URL);
  const balance = await provider.getBalance(address);
  return ethers.utils.formatEther(balance);
}

interface RequestBalanceResponse {
  balance: string;
  status: string;
}

interface TokenChainPair {
  tokenTicker: string;
  rollupContract: address;
}

export async function fetchContracts(): Promise<{ chainA: string; chainB: string }> {
  const response = await fetch(`${process.env.REACT_APP_RPC_URL}/contracts`);
  if (!response.ok) {
    throw new Error(`Failed to fetch contracts! Status: ${response.status}`);
  }
  const data = await response.json();
  return {
    chainA: data.chain_a.spvm.address,
    chainB: data.chain_b.spvm.address,
  };
}

export async function getBalancesForAllTokens(walletAddress: address, tokensAndChains: TokenChainPair[]): Promise<RequestBalanceResponse[]> {
  try {
    const balancePromises = tokensAndChains.map(({ tokenTicker, rollupContract }) =>
      getBalanceForToken(walletAddress, tokenTicker, rollupContract)
    );

    // Wait for all the promises to resolve
    const balances = await Promise.all(balancePromises);

    return balances; // Return the array of balance responses
  } catch (error) {
    // Handle any errors that occurred during the fetch operations
    console.error('Error fetching balances for all tokens:', error);
    throw error;
  }
}

export async function getBalanceForToken(walletAddress: address, tokenTicker: string, rollupContract: address): Promise<RequestBalanceResponse> {
    try {
      // Perform the fetch request
      const response = await fetch(`${process.env.REACT_APP_API_URL}/request_balance?address=${walletAddress}&token_ticker=${tokenTicker}&rollup_contract=${rollupContract}`, {
          method: 'GET', // Since curl uses GET by default
          headers: {
              'Content-Type': 'application/json' // Optional: Define headers if needed
          }
      });

      // Check if the response is ok (status code 200-299)
      if (!response.ok) {
          throw new Error(`HTTP error! Status: ${response.status}`);
      }

      // Parse the JSON response
      const data = await response.json();

      data.tokenTicker = tokenTicker
      // Return the parsed data
      return data;
  } catch (error) {
      // Log and rethrow the error for further handling
      console.error('Error fetching balance:', error);
      throw error;
  }
}

//
// Tx Generation methods
//

type address = string;

interface TransactionContent {
  from: address;
  txType: number;
  txParam: string;
  nonce: number;
}

export interface Transaction {
  txContent: TransactionContent;
  transactionHash: string;
  signature?: string;
  specialTx?: string;
}

interface MintTransactionParams {
  tokenTicker: string;
  owner: address;
  supply: number;
}

interface TransferTransactionParams {
  tokenTicker: string;
  to: address;
  amount: number;
}

export const MINT_TX_TYPE = 0x00;
export const TRANSFER_TX_TYPE = 0x01;

export const createMintTransaction = async (
  owner: address,
  supply: number,
  tokenTicker: string,
  fromAddress: address,
  nonce: number,
): Promise<Transaction> => {
  const abiEncoder = new ethers.utils.AbiCoder();

  const txParam = abiEncoder.encode(
    ["string", "address", "uint256"],
    [tokenTicker, owner, supply],
  );

  const txContent: TransactionContent = {
    from: fromAddress,
    txType: MINT_TX_TYPE,
    txParam: txParam,
    nonce: nonce,
  };

  const serializedTxContent = abiEncoder.encode(
    ["address", "uint8", "bytes", "uint32"],
    [txContent.from, txContent.txType, txContent.txParam, txContent.nonce],
  );

  const transactionHash = ethers.utils.keccak256(serializedTxContent);

  const transaction: Transaction = {
    txContent: txContent,
    transactionHash: transactionHash,
    signature: undefined, // This will be filled in when the transaction is signed
  };

  return transaction;
};

export const createTransferTransaction = async (
  to: address,
  amount: number,
  tokenTicker: string,
  fromAddress: address,
  nonce: number,
): Promise<Transaction> => {
  const abiEncoder = new ethers.utils.AbiCoder();

  const txParam = abiEncoder.encode(
    ["tuple(string,address,uint16)"],
    [[tokenTicker, to, amount]],
  );

  console.log("txParam")
  console.log(txParam);

  const txContent: TransactionContent = {
    from: fromAddress,
    txType: TRANSFER_TX_TYPE,
    txParam: txParam,
    nonce: nonce,
  };

  const serializedTxContent = abiEncoder.encode(
    ["tuple(address,uint8,bytes,uint32)"],
    [[txContent.from, txContent.txType, txContent.txParam, txContent.nonce]],
  );

  console.log("serialized")
  console.log(serializedTxContent);

  const transactionHash = ethers.utils.keccak256(serializedTxContent);

  const transaction: Transaction = {
    txContent: txContent,
    transactionHash: transactionHash,
    signature: undefined, // This will be filled in when the transaction is signed
    specialTx: serializedTxContent
  };

  return transaction;
};

const stringifyTxData = (transaction: Transaction): string => {
  let retVal = "";
  try {
    const jsonString = JSON.stringify(transaction.txContent);
    const uintArray = new TextEncoder().encode(jsonString);
    const hexTxData = ethers.utils.hexlify(uintArray);
    retVal = hexTxData;
  } catch (err) {
    console.error(err);
  }
  return retVal;
};

export const signTransaction = async (
  transaction: Transaction,
  privateKey: string,
  to: address,
): Promise<Transaction> => {
  const wallet = new ethers.Wallet(privateKey);

  const pk = new ethers.utils.SigningKey(privateKey);
  const signedHash = await pk.signDigest(transaction.transactionHash);

  transaction.signature = ethers.utils.joinSignature(signedHash);

  console.log("Signature: " + transaction.signature);

  return transaction;
};

export const storeTransaction = (
  userAddress: address,
  transaction: Transaction,
) => {
  const existingTransactions = localStorage.getItem(userAddress);
  const transactions = existingTransactions
    ? JSON.parse(existingTransactions)
    : [];
  transactions.push(transaction);
  localStorage.setItem(userAddress, JSON.stringify(transactions));
};

export const getTransactions = (userAddress: address) => {
  const transactions = localStorage.getItem(userAddress);
  return transactions ? JSON.parse(transactions) : [];
};
