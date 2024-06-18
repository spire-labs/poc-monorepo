import React, { useState, useEffect } from "react";
import {
  Badge,
  Box,
  Flex,
  List,
  ListItem,
  Modal,
  ModalOverlay,
  ModalContent,
  ModalHeader,
  ModalCloseButton,
  ModalBody,
  useDisclosure,
  Button,
} from "@chakra-ui/react";
import { CheckCircleIcon } from "@chakra-ui/icons";
import { Transaction } from "./utils/utils";

interface TransactionListProps {
  transactions: Transaction[];
}

export const TransactionList: React.FC<TransactionListProps> = ({
  transactions,
}) => {
  const { isOpen, onOpen, onClose } = useDisclosure();
  const [selectedTx, setSelectedTx] = useState<Transaction | null>(null);
  const [statuses, setStatuses] = useState<{ [key: string]: string }>({});

  const fetchStatus = async (txHash: string) => {
    try {
      const response = await fetch(
        `${process.env.REACT_APP_API_URL}/preconfirmation_status?tx_hash=${txHash}`
      );
      const data = await response.json();
      return data.data.status;
    } catch (error) {
      console.error("Error fetching status:", error);
      return null;
    }
  };

  const updateStatuses = async () => {
    const newStatuses = { ...statuses };

    for (const tx of transactions) {
      if (!newStatuses[tx.transactionHash] || newStatuses[tx.transactionHash] === "PENDING") {
        // const status = await fetchStatus(tx.transactionHash);
        // if (status) {
        //   newStatuses[tx.transactionHash] = status;
        // }
      }
    }

    setStatuses(newStatuses);
  };

  useEffect(() => {
    updateStatuses();

    const interval = setInterval(updateStatuses, 1000);
    return () => clearInterval(interval);
  }, [transactions]);

  const openModal = (tx: Transaction) => {
    setSelectedTx(tx);
    onOpen();
  };

  return (
    <Box>
      <List spacing={3}>
        {transactions.map((tx, index) => (
          <ListItem
            key={index}
            p={4}
            boxShadow="md"
            borderRadius="md"
            _hover={{ bg: "gray.100", cursor: "pointer" }}
            onClick={() => openModal(tx)}
            display={'flex'}
          >
            <Box>
              Transaction Type: {tx.txContent.txType} - Hash:{" "}
              {`${tx.transactionHash.slice(0, 15)}...`}
            </Box>
            {statuses[tx.transactionHash] === "PENDING" ? (
              <Flex alignItems="baseline" height="100%">
                <Badge colorScheme="gray">
                  <CheckCircleIcon mr={2} /> Pending
                </Badge>
              </Flex>
            ) : (
              <Flex alignItems="baseline" height="100%">
                <Badge colorScheme="green">
                  <CheckCircleIcon mr={2} /> Preconfirmed
                </Badge>
              </Flex>
            )}
          </ListItem>
        ))}
      </List>

      <Modal isOpen={isOpen} onClose={onClose}>
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>
            Transaction Details{" "}
            {statuses[selectedTx?.transactionHash || ""] === "PENDING" ? (
              <Flex alignItems="baseline" height="100%">
                <Badge colorScheme="gray">
                  <CheckCircleIcon mr={2} /> Pending
                </Badge>
              </Flex>
            ) : (
              <Flex alignItems="baseline" height="100%">
                <Badge colorScheme="green">
                  <CheckCircleIcon mr={2} /> Preconfirmed
                </Badge>
              </Flex>
            )}
          </ModalHeader>
          <ModalCloseButton />
          <ModalBody>
            <pre style={{ whiteSpace: 'pre-wrap', overflowWrap: 'break-word' }}>
              {JSON.stringify(selectedTx, null, 2)}
            </pre>
          </ModalBody>
        </ModalContent>
      </Modal>
    </Box>
  );
};
