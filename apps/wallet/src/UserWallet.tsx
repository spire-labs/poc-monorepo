import React, { useEffect, useState } from "react";
import { ethers } from "ethers";
import {
  createMintTransaction,
  createTransferTransaction,
  fetchContracts,
  getBalance,
  getBalancesForAllTokens,
  getTransactions,
  signTransaction,
  storeTransaction,
  Transaction,
} from "./utils/utils";
import {
  Box,
  Button,
  Flex,
  FormControl,
  FormLabel,
  HStack,
  IconButton,
  Image,
  Input,
  Modal,
  ModalOverlay,
  ModalContent,
  ModalHeader,
  ModalFooter,
  ModalBody,
  ModalCloseButton,
  useDisclosure,
  Select,
  Stack,
  Tabs,
  TabList,
  TabPanels,
  Tab,
  TabPanel,
  TabIndicator,
  Text,
  useClipboard,
  useToast,
  VStack,
} from "@chakra-ui/react";
import { FaCopy, FaPlus, FaArrowRight, FaExchangeAlt } from "react-icons/fa";
import { TransactionList } from "./TransactionList";
import Confetti from 'react-confetti';

const chains = [
  { id: "11", name: "Chain A" },
  { id: "22", name: "Chain B" },
];

interface TokenChainPair {
  tokenTicker: string;
  rollupContract: string;
}

interface Token {
  symbol: string;
  name: string;
  logo: string;
  balance: string; // Initial balance is a string for display
}

export const UserWallet: React.FC = () => {
  const [tokens, setTokens] = useState<Token[]>([
    {
      symbol: "RAIN",
      name: "Rain coin",
      logo: `${process.env.PUBLIC_URL}/rain_coin.png`,
      balance: "0.0",
    },
    // {
    //   symbol: "INFINITY",
    //   name: "Infinity token",
    //   logo: `${process.env.PUBLIC_URL}/infinity_coin.png`,
    //   balance: "0.0",
    // },
    {
      symbol: "QUEEN",
      name: "Queen Token",
      logo: `${process.env.PUBLIC_URL}/queen_coin.png`,
      balance: "0.0",
    },
  ]);
  const [balance, setBalance] = useState<string>("0");
  const [selectedChain, setSelectedChain] = useState<string>(chains[0].id);
  const [selectedToken, setSelectedToken] = useState<string>(tokens[0].symbol);
  const [tokenBalances, setTokenBalances] = useState<Record<string, string>>(
    {},
  );
  const [tokenBalances2, setTokenBalances2] = useState<Record<string, string>>(
    {},
  );
  // For the PoC Demo, we use a hardcoded address
  const [address, setAddress] = useState<string>("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
  const [currentNonce, setCurrentNonce] = useState(1);
  const { hasCopied, onCopy } = useClipboard(address);
  const [existingTransactions, setExistingTransactions] = useState<
    Transaction[]
  >([]);
  const toast = useToast();
  const { isOpen, onOpen, onClose } = useDisclosure();
  const [tabIndex, setTabIndex] = useState(0); // 0 for Transfer, 1 for Mint
  const handleTabsChange = (index: number) => {
    setTabIndex(index);
  };



  // state for transfers
  const [transferTxTo, setTransferTxTo] = useState("");
  const [transferTxChain, setTransferTxChain] = useState("Chain A");
  const [transferTxValue, setTransferTxValue] = useState(0);

  const [showConfetti, setShowConfetti] = useState(false);

  const [chainAContract, setChainAContract] = useState<string | null>(null);
  const [chainBContract, setChainBContract] = useState<string | null>(null);
  const [balancesLoading, setBalancesLoading] = useState(false);
  const [balancesError, setBalancesError] = useState('');
  const [balances, setBalances] = useState<any>(null);

  const tokenPairs: TokenChainPair[] = [
    { tokenTicker: "RAIN", rollupContract: chainAContract ?? "" },
    { tokenTicker: "QUEEN", rollupContract: chainBContract ?? "" },
  ];

  useEffect(() => {
    const fetchAndSetContracts = async () => {
      try {
        const { chainA, chainB } = await fetchContracts();
        setChainAContract(chainA);
        setChainBContract(chainB);
      } catch (err) {
        console.error('Failed to fetch contracts.');
      }
    };
    fetchAndSetContracts();
  }, []);
  
  useEffect(() => {
    // fetch balances once on load
    if (chainAContract && chainBContract) {
      const fetchBalances = async () => {
        try {
          setBalancesLoading(true);
          setBalancesError('');
          const fetchedBalances = await getBalancesForAllTokens(address, tokenPairs);
          setBalances(fetchedBalances);

        } catch (err) {
          setBalancesError('Failed to fetch balances.');
        } finally {
          setBalancesLoading(false);
        }
      };

      fetchBalances();
    }
  }, [chainAContract, chainBContract, address]);

  useEffect(() => {
    // Check for address in local storage when the component mounts
    const storedAddress = localStorage.getItem("address");
    if (storedAddress) {
      setAddress(storedAddress);

      const txs = getTransactions(storedAddress);
      setExistingTransactions(txs);
    } else {
      // For the PoC demo, use a hardcoded Anvil default address and pk. These are default test values from anvil, no real funds are at risk here.
      localStorage.setItem("privateKey", "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6");
      localStorage.setItem("address", "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
      localStorage.setItem("nonce", "0");
    }

    const storedNonce = localStorage.getItem("nonce");
    if (storedNonce) {
      //setCurrentNonce(Number(storedNonce));
      // for the demo, always zero out the nonce
      setCurrentNonce(0);
    }
  }, []);

  // Update tokens with fetched balances
  useEffect(() => {
    console.log('balances', balances)
    if (balances?.length > 0) {
      const updatedTokens = tokens.map(token => {
        const matchingBalance = balances.find((balance:any) => balance.tokenTicker === token.symbol);
        return matchingBalance ? { ...token, balance: matchingBalance.balance.toString()} : token;
      });
      console.log('UPDATED TOKEN BALANCES', updatedTokens)
      setTokens(updatedTokens);
      const updatedBalances: Record<string, string> = {};
      updatedTokens.forEach((token) => {
        updatedBalances[token.symbol] = token.balance;
      });
      setTokenBalances(updatedBalances);
      // const updatedBalances2: Record<string, string> = {};
      // updatedTokens.forEach((token) => {
      //   updatedBalances2[token.symbol] = token.balance;
      // });
      // updatedBalances['RAIN'] = '25'//'100'
      // updatedBalances2['RAIN'] = '75'//'0'
      setTokenBalances(updatedBalances);
      // setTokenBalances2(updatedBalances2);
    }
  }, [balances]);

  // const forceUpdateBalances = (u: any) => {
  //   let balances = [
  //     {balance: "25", status: "OK", tokenTicket: "RAIN"},
  //     {balance: "200", status: "OK", tokenTicket: "QUEEN"},
  //   ]
  //   // if (u) {
  //   //   balances = u;
  //   // }
  //   const updatedTokens = tokens.map(token => {
  //     const matchingBalance = balances.find((balance:any) => balance.tokenTicker === token.symbol);
  //     return matchingBalance ? { ...token, balance: matchingBalance.balance.toString()} : token;
  //   });
  //   console.log('UPDATED TOKEN BALANCES', updatedTokens)
  //   setTokens(updatedTokens);
  //   const updatedBalances: Record<string, string> = {};
  //   updatedTokens.forEach((token) => {
  //     updatedBalances[token.symbol] = token.balance;
  //   });
  //   // setTokenBalances(updatedBalances);
  //   const updatedBalances2: Record<string, string> = {};
  //   updatedTokens.forEach((token) => {
  //     updatedBalances2[token.symbol] = token.balance;
  //   });
  //   // updatedBalances['RAIN'] = '25'
  //   // updatedBalances2['RAIN'] = '75'
  //   updatedBalances['RAIN'] = '10'
  //   updatedBalances2['RAIN'] = '75'
  //   updatedBalances2['QUEEN'] = '215'
  //   setTokenBalances(updatedBalances);
  //   setTokenBalances2(updatedBalances2);
  // }

  const generateWallet = () => {
    const storedAddress = localStorage.getItem("address");
    // For now, only generate a new wallet if an address doesn't already exist
    if (!storedAddress) {
      const wallet = ethers.Wallet.createRandom();
      localStorage.setItem("privateKey", wallet.privateKey);
      localStorage.setItem("address", wallet.address);
      localStorage.setItem("nonce", "0");
      setAddress(wallet.address);
      toast({
        title: "New Wallet Created",
        description: `Address: ${wallet.address}`,
        status: "success",
        duration: 9000,
        isClosable: true,
      });
    } else {
      toast({
        title: "Wallet Already Exists",
        description: "Using the existing wallet address.",
        status: "info",
        duration: 9000,
        isClosable: true,
      });
    }
  };

  const truncateAddress = (address: string) => {
    return `${address.substring(0, 6)}...${address.substring(address.length - 4)}`;
  };

  const checkBalance = async () => {
    const fetchedBalances = await getBalancesForAllTokens(address, tokenPairs);
    setBalances(fetchedBalances);
  };

  const handleTokenChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedToken(event.target.value);
  };

  const handleChainChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedChain(event.target.value);
    console.log(`new chain ${event.target.value}`)
  };
  const handleTxChainChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setTransferTxChain(event.target.value);
  };

  return (
    <Box
      width="full"
      maxWidth="400px"
      mx="auto"
      p={5}
      boxShadow="md"
      borderRadius="lg"
    >
      <VStack spacing={4}>
        <Flex justifyContent="space-between" alignItems="center" width="full">
          <Flex alignItems="center">
            <Text fontSize="md" fontWeight="bold">
              {address ? truncateAddress(address) : "No Wallet"}
            </Text>
            {address && (
              <IconButton
                aria-label="Copy address"
                icon={<FaCopy />}
                onClick={onCopy}
                size="sm"
                isRound={true}
              />
            )}
          </Flex>
          {/* <Button leftIcon={<FaPlus />} onClick={generateWallet} size="sm">
            New
          </Button> */}
        </Flex>
        <HStack width="full" justifyContent="space-between">
  {/* Container for the tokens */}
  <Flex flex="1" alignItems="center">
    {tokens.map((token) => (
      <Box key={token.symbol} textAlign="center">
        {selectedToken === token.symbol && (
          <Flex alignItems="center">
            <Image
              src={token.logo}
              boxSize="30px"
              borderRadius={25}
              alt={`${token.name} Logo`}
              mr={2} // Adds margin to the right of the image
            />
            <Text fontSize="2xl">
              {selectedChain == '11' ? tokenBalances[token.symbol] : tokenBalances2[token.symbol]} {token.symbol}
            </Text>
          </Flex>
        )}
      </Box>
    ))}
  </Flex>

  {/* Select component pinned to the right */}
  <Select onChange={handleTokenChange} value={selectedToken} w={100} flexShrink="0">
    {tokens.map((token) => (
      <option key={token.symbol} value={token.symbol}>
        {token.name}
      </option>
    ))}
  </Select>
</HStack>
        {/* <HStack width="full" justifyContent="space-between">
          {tokens.map((token) => (
            <Box key={token.symbol} textAlign="center">
              {selectedToken === token.symbol && (
                <Flex alignItems="center" justifyContent="space-between">
                  <Image
                    src={token.logo}
                    boxSize="30px"
                    borderRadius={25}
                    alt={`${token.name} Logo`}
                  />
                  <Text fontSize="2xl">
                    {selectedChain == '11' ? tokenBalances[token.symbol] : tokenBalances2[token.symbol]} {token.symbol}
                  </Text>
                </Flex>
              )}
            </Box>
          ))}
          <Select onChange={handleTokenChange} value={selectedToken} w={100}>
            {tokens.map((token) => (
              <option key={token.symbol} value={token.symbol}>
                {token.name}
              </option>
            ))}
          </Select>
        </HStack> */}
        <Button
          leftIcon={<FaArrowRight />}
          onClick={checkBalance}
          size="sm"
          width="full"
        >
          Check Balance
        </Button>
        <FormControl>
          <FormLabel>Chain</FormLabel>
          <Select onChange={handleChainChange} value={selectedChain}>
            {chains.map((chain) => (
              <option key={chain.id} value={chain.id}>
                {chain.name}
              </option>
            ))}
          </Select>
        </FormControl>

        <Tabs
          isFitted
          variant="unstyled"
          onChange={handleTabsChange}
          index={tabIndex}
          w="100%"
        >
          <TabList mb="1em">
            <Tab>Transfer</Tab>
            <Tab>Swap</Tab>
            <Tab>History</Tab>
          </TabList>
          <TabIndicator
            mt="-1.5px"
            height="2px"
            bg="#5ac8fa"
            borderRadius="1px"
          />
          <TabPanels>
            <TabPanel paddingLeft={0} paddingRight={0}>
              <FormControl>
                <FormLabel htmlFor="to">To</FormLabel>
                <Input
                  id="to"
                  name="to"
                  value={transferTxTo}
                  onChange={(e) => setTransferTxTo(e.target.value)}
                  placeholder="Address"
                />
              </FormControl>
              <FormControl>
                <FormLabel htmlFor="chain">Chain</FormLabel>
                <Select onChange={handleTxChainChange} value={transferTxChain}>
                  {chains.map((chain) => (
                    <option key={chain.id} value={chain.id}>
                      {chain.name}
                    </option>
                  ))}
                </Select>
              </FormControl>
              <FormControl mt={4}>
                <FormLabel htmlFor="amount">Amount</FormLabel>
                <Input
                  id="amount"
                  value={transferTxValue}
                  onChange={(e) => setTransferTxValue(Number(e.target.value))}
                  placeholder={`${tokenBalances[selectedToken]} ${selectedToken}`}
                />
              </FormControl>
            </TabPanel>
            <TabPanel paddingLeft={0} paddingRight={0}>
              <FormControl>
                <FormLabel htmlFor="to">To</FormLabel>
                <Input
                  id="to"
                  name="to"
                  value={transferTxTo}
                  onChange={(e) => setTransferTxTo(e.target.value)}
                  placeholder="Address"
                />
              </FormControl>
              <FormLabel htmlFor="to">Token</FormLabel>
              <Select /*onChange={handleTokenChange} value={selectedToken}*/ w={150}>
            {tokens.map((token) => (
              <option key={token.symbol} value={token.symbol}>
                {token.name}
              </option>
            ))}
          </Select>
              <FormControl>
                <FormLabel htmlFor="chain">Chain</FormLabel>
                <Select onChange={handleTxChainChange} value={transferTxChain}>
                  {chains.map((chain) => (
                    <option key={chain.id} value={chain.id}>
                      {chain.name}
                    </option>
                  ))}
                </Select>
              </FormControl>
              <FormControl mt={4}>
                <FormLabel htmlFor="amount">Amount</FormLabel>
                <Input
                  id="amount"
                  value={transferTxValue}
                  onChange={(e) => setTransferTxValue(Number(e.target.value))}
                  placeholder={`${tokenBalances[selectedToken]} ${selectedToken}`}
                />
              </FormControl>
            </TabPanel>
            <TabPanel paddingLeft={0} paddingRight={0}>
              <TransactionList transactions={existingTransactions} />
            </TabPanel>
          </TabPanels>
        </Tabs>

        <Button
          leftIcon={<FaExchangeAlt />}
          onClick={onOpen}
          size="sm"
          width="full"
        >
          Send Transaction
        </Button>
        {showConfetti && <Confetti width={window.innerWidth} height={window.innerHeight} />}
        {/* Modal for sending transaction */}
        <Modal isOpen={isOpen} onClose={onClose}>
          <ModalOverlay />
          <ModalContent>
            <ModalHeader>Send Transaction</ModalHeader>
            <ModalCloseButton />
            <ModalBody>
              <Text>Sign and send your transaction</Text>
            </ModalBody>

            <ModalFooter>
              <Button mr={3} onClick={onClose}>
                Close
              </Button>
              <Button
                variant="ghost"
                _hover={{ bg: "#5ac8fa" }}
                onClick={async () => {
                  let tx;
                  let to;
                  // if (tabIndex === 0) {
                    tx = await createTransferTransaction(
                      transferTxTo,
                      transferTxValue,
                      selectedToken,
                      address,
                      currentNonce, // TODO: reactivate NONCEs
                    );
                    console.log("Nonce: ", currentNonce);
                    to = transferTxTo;
                  const privateKey = localStorage.getItem("privateKey");
                  if (!privateKey) {
                    console.error(
                      "ERROR: Attempted to sign tx without private key",
                    );
                    return;
                  }
                  const signedTx = await signTransaction(tx, privateKey, to);
                  console.log("DEBUG: Transaction", signedTx);

                  if (!signedTx || !signedTx.specialTx) {
                    console.log("ERR: Insufficient request body");
                    return;
                  }
                  // Construct the request body
                  const requestBody = {
                    tx_hash: signedTx.transactionHash,
                    tx_content: signedTx.specialTx,
                    signature: signedTx.signature,
                    chain: transferTxChain === chains[0]?.name ? "chain_a" : "chain_b"
                  };

                  console.log("----------------GATEWAY API REQUEST----------------")
                  console.log("Request body being sent to gateway: ", requestBody);
                  console.log("Stringified request body: ", JSON.stringify(requestBody));

                  // Send the request to the gateway
                  const response = await fetch(`${process.env.REACT_APP_API_URL}/request_preconfirmation`, {
                    method: 'POST',
                    headers: {
                      'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(requestBody),
                  });

                  if (!response.ok) {
                    console.error(`HTTP error! status: ${response.status}`);
                    toast({
                      title: "Preconfirmation request was unsuccessful.",
                      status: "error",
                      duration: 5000,
                      isClosable: true,
                    });
                    return;
                  }

                  // For now, a 200 with no body indicates that the response was successful
                  toast({
                    title: "Preconfirmation request was successful.",
                    status: "success",
                    duration: 5000,
                    isClosable: true,
                  });

                  setShowConfetti(true);
                  setTimeout(() => setShowConfetti(false), 5000);

                  // TODO: preemptively update nonce for now
                  // This should be updated once we get confirmation from gateway
                  const newNonce = currentNonce + 1;
                  setCurrentNonce(newNonce);
                  localStorage.setItem("nonce", String(newNonce));

                  storeTransaction(address, signedTx);
                  setExistingTransactions([
                    ...existingTransactions,
                    signedTx as any,
                  ]);
                  // forceUpdateBalances({});
                  onClose();
                }}
              >
                Sign and Send Transaction
              </Button>
            </ModalFooter>
          </ModalContent>
        </Modal>
      </VStack>
    </Box>
  );
};
