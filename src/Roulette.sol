// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {VRFConsumerBaseV2Plus} from "@chainlink/contracts@1.2.0/src/v0.8/vrf/dev/VRFConsumerBaseV2Plus.sol";
import {VRFV2PlusClient} from "@chainlink/contracts@1.2.0/src/v0.8/vrf/dev/libraries/VRFV2PlusClient.sol";

interface IERC20 {
    function name() external pure returns (string memory);

    function symbol() external pure returns (string memory);

    function decimals() external pure returns (uint8);

    function totalSupply() external view returns (uint256);

    function balanceOf(address owner) external view returns (uint256);

    function transfer(address to, uint256 value) external returns (bool);

    function transferFrom(address from, address to, uint256 value) external returns (bool);

    function approve(address spender, uint256 value) external returns (bool);

    function allowance(address owner, address spender) external view returns (uint256);

      function mint(uint256 value) external;

    function mintTo(address to, uint256 value) external;

    function burn(uint256 value) external;
}



/**
 * @notice A Chainlink VRF consumer which uses randomness to mimic the rolling
 * of a 20 sided dice
 */

/**
 * Request testnet LINK and ETH here: https://faucets.chain.link/
 * Find information on LINK Token Contracts and get the latest ETH and LINK faucets here: https://docs.chain.link/docs/link-token-contracts/
 */

/**
 * THIS IS AN EXAMPLE CONTRACT THAT USES HARDCODED VALUES FOR CLARITY.
 * THIS IS AN EXAMPLE CONTRACT THAT USES UN-AUDITED CODE.
 * DO NOT USE THIS CODE IN PRODUCTION.
 */

contract LuckyChain is VRFConsumerBaseV2Plus {
    uint256 private constant ROLL_IN_PROGRESS = 42;

    uint256[37] private  ROULETTE_NUMBERS = [
  0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8, 23, 10, 5,
  24, 16, 33, 1, 20, 14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26
];

    // Your subscription ID.
    uint256 public s_subscriptionId;

    // Sepolia coordinator. For other networks,
    // see https://docs.chain.link/vrf/v2-5/supported-networks#configurations
    address public vrfCoordinator = 0x5CE8D5A2BC84beb22a398CCA51996F7930313D61;

    // The gas lane to use, which specifies the maximum gas price to bump to.
    // For a list of available gas lanes on each network,
    // see https://docs.chain.link/vrf/v2-5/supported-networks#configurations
    bytes32 public s_keyHash =
        0x1770bdc7eec7771f7ba4ffd640f34260d7f095b79c92d34a5b2551d6f6cfd2be;

    // Depends on the number of requested values that you want sent to the
    // fulfillRandomWords() function. Storing each word costs about 20,000 gas,
    // so 40,000 is a safe default for this example contract. Test and adjust
    // this limit based on the network that you select, the size of the request,
    // and the processing of the callback request in the fulfillRandomWords()
    // function.
    uint32 public callbackGasLimit = 40000;

    // The default is 3, but you can set this higher.
    uint16 public requestConfirmations = 3;

    // For this example, retrieve 1 random value in one request.
    // Cannot exceed VRFCoordinatorV2_5.MAX_NUM_WORDS.
    uint32 public numWords = 1;

    // Stylus Token
   IERC20 public betToken;

    // Struct Bet
     struct Bet {
        address player;
        uint256 amount;
        uint256 number;
    }

    mapping(uint256 => Bet) public bets;
    mapping(address => uint256) public pendingWithdrawals;
    

    // map rollers to requestIds
    mapping(uint256 => address) private s_rollers;
    // map vrf results to rollers
    mapping(address => uint256) private s_results;



    event BetPlaced(address indexed player, uint256 amount, uint256 number);
    event WheelSpun(uint256 indexed requestId, address indexed player);
    event BetResult(address indexed player, uint256 betNumber, uint256 luckyNumber, uint256 payout);

    /**
     * @notice Constructor inherits VRFConsumerBaseV2Plus
     *
     * @dev NETWORK: Arbitrum
     *
     * @param subscriptionId subscription ID that this consumer contract can use
     */
    constructor(uint256 subscriptionId, address _betToken) VRFConsumerBaseV2Plus(vrfCoordinator) {
        s_subscriptionId = subscriptionId;
        betToken = IERC20(_betToken);
    }


    function placeBet(uint256 _number, uint256 _amount) external {
        require(_number >= 0 && _number <= 36, "Invalid number");
        require(_amount > 0, "Bet amount must be greater than 0");
        require(betToken.transferFrom(msg.sender, address(this), _amount), "Token transfer failed");

        uint256 requestId  = s_vrfCoordinator.requestRandomWords(
            VRFV2PlusClient.RandomWordsRequest({
                keyHash: s_keyHash,
                subId: s_subscriptionId,
                requestConfirmations: requestConfirmations,
                callbackGasLimit: callbackGasLimit,
                numWords: numWords,
                extraArgs: VRFV2PlusClient._argsToBytes(
                    // Set nativePayment to true to pay for VRF requests with Sepolia ETH instead of LINK
                    VRFV2PlusClient.ExtraArgsV1({nativePayment: false})
                )
            })
        );

        bets[requestId] = Bet({
            player: msg.sender,
            amount: _amount,
            number: _number
        });

        emit BetPlaced(msg.sender, _amount, _number);
        emit WheelSpun(requestId, msg.sender);
    }

    /**
     * @notice Callback function used by VRF Coordinator to return the random number to this contract.
     *
     * @dev Some action on the contract state should be taken here, like storing the result.
     * @dev WARNING: take care to avoid having multiple VRF requests in flight if their order of arrival would result
     * in contract states with different outcomes. Otherwise miners or the VRF operator would could take advantage
     * by controlling the order.
     * @dev The VRF Coordinator will only send this function verified responses, and the parent VRFConsumerBaseV2
     * contract ensures that this method only receives randomness from the designated VRFCoordinator.
     *
     * @param requestId uint256
     * @param randomWords  uint256[] The random result returned by the oracle.
     */
    function fulfillRandomWords(
        uint256 requestId,
        uint256[] calldata randomWords
    ) internal override {
        Bet memory bet = bets[requestId];
        uint256 luckyNumber = ROULETTE_NUMBERS[randomWords[0] % 37];

        uint256 payout = 0;
        if (bet.number == luckyNumber) {
            payout = bet.amount * 5;
            pendingWithdrawals[bet.player] += payout;
        } else {
            // If the bet doesn't win, the tokens are kept by the contract
            // No action needed as the tokens were already transferred in placeBet()
        }

        emit BetResult(bet.player, bet.number, luckyNumber, payout);
        delete bets[requestId];
    }

    function withdraw() external {
        uint256 amount = pendingWithdrawals[msg.sender];
        require(amount > 0, "No funds to withdraw");

        pendingWithdrawals[msg.sender] = 0;
        require(betToken.transfer(msg.sender, amount), "Token transfer failed");
    }

    // Function to allow the owner to withdraw accumulated tokens
    function ownerWithdraw(uint256 _amount) external onlyOwner {
        require(betToken.balanceOf(address(this)) >= _amount, "Insufficient balance");
        require(betToken.transfer(owner(), _amount), "Token transfer failed");
    }

    function updateSubscriptionId(uint64 _subscriptionId) external onlyOwner {
        s_subscriptionId = _subscriptionId;
    }

    function updateCallbackGasLimit(uint32 _callbackGasLimit) external onlyOwner {
        callbackGasLimit = _callbackGasLimit;
    }



}
