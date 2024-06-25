import { Box, Text } from "@chakra-ui/react";
import { UserWallet } from "./UserWallet";

export const Home = () => (
  <Box
    p={4}
    display="flex"
    flexDirection="column"
    justifyContent="center"
    alignItems="center"
    height="100vh"
  >
    <Text fontSize="2xl">Welcome to Spire Wallet</Text>
    <UserWallet />
  </Box>
);
