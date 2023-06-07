export type OpenbookV2 = {
  "version": "0.1.0",
  "name": "openbook_v2",
  "instructions": [
    {
      "name": "createMarket",
      "docs": [
        "Create a [`Market`](crate::state::Market) for a given token pair."
      ],
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Accounts are initialised by client,",
            "anchor discriminator is set first when ix exits,"
          ]
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "quoteMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "oracle",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "marketIndex",
          "type": "u32"
        },
        {
          "name": "name",
          "type": "string"
        },
        {
          "name": "oracleConfig",
          "type": {
            "defined": "OracleConfigParams"
          }
        },
        {
          "name": "quoteLotSize",
          "type": "i64"
        },
        {
          "name": "baseLotSize",
          "type": "i64"
        },
        {
          "name": "makerFee",
          "type": "f32"
        },
        {
          "name": "takerFee",
          "type": "f32"
        },
        {
          "name": "feePenalty",
          "type": "u64"
        },
        {
          "name": "collectFeeAdmin",
          "type": "publicKey"
        },
        {
          "name": "openOrdersAdmin",
          "type": {
            "option": "publicKey"
          }
        },
        {
          "name": "consumeEventsAdmin",
          "type": {
            "option": "publicKey"
          }
        },
        {
          "name": "closeMarketAdmin",
          "type": {
            "option": "publicKey"
          }
        }
      ]
    },
    {
      "name": "initOpenOrders",
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "accountNum",
          "type": "u32"
        },
        {
          "name": "openOrdersCount",
          "type": "u8"
        }
      ]
    },
    {
      "name": "placeOrder",
      "docs": [
        "Place an order.",
        "",
        "Different types of orders have different effects on the order book,",
        "as described in [`PlaceOrderType`](crate::state::PlaceOrderType).",
        "",
        "`price_lots` refers to the price in lots: the number of quote lots",
        "per base lot. It is ignored for `PlaceOrderType::Market` orders.",
        "",
        "`expiry_timestamp` is a unix timestamp for when this order should",
        "expire. If 0 is passed in, the order will never expire. If the time",
        "is in the past, the instruction is skipped. Timestamps in the future",
        "are reduced to now + 65,535s.",
        "",
        "`limit` determines the maximum number of orders from the book to fill,",
        "and can be used to limit CU spent. When the limit is reached, processing",
        "stops and the instruction succeeds."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "openOrdersAdmin",
          "isMut": false,
          "isSigner": true,
          "isOptional": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenDepositAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "oracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "priceLots",
          "type": "i64"
        },
        {
          "name": "maxBaseLots",
          "type": "i64"
        },
        {
          "name": "maxQuoteLotsIncludingFees",
          "type": "i64"
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "PlaceOrderType"
          }
        },
        {
          "name": "selfTradeBehavior",
          "type": {
            "defined": "SelfTradeBehavior"
          }
        },
        {
          "name": "expiryTimestamp",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u8"
        }
      ],
      "returns": {
        "option": "u128"
      }
    },
    {
      "name": "placeOrderPegged",
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "openOrdersAdmin",
          "isMut": false,
          "isSigner": true,
          "isOptional": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenDepositAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "oracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "priceOffsetLots",
          "type": "i64"
        },
        {
          "name": "pegLimit",
          "type": "i64"
        },
        {
          "name": "maxBaseLots",
          "type": "i64"
        },
        {
          "name": "maxQuoteLotsIncludingFees",
          "type": "i64"
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "PlaceOrderType"
          }
        },
        {
          "name": "selfTradeBehavior",
          "type": {
            "defined": "SelfTradeBehavior"
          }
        },
        {
          "name": "expiryTimestamp",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u8"
        },
        {
          "name": "maxOracleStalenessSlots",
          "type": "i32"
        }
      ],
      "returns": {
        "option": "u128"
      }
    },
    {
      "name": "placeTakeOrder",
      "docs": [
        "Place an order that shall take existing liquidity off of the book, not",
        "add a new order off the book.",
        "",
        "This type of order allows for instant token settlement for the taker."
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenDepositAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenReceiverAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "oracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "openOrdersAdmin",
          "isMut": false,
          "isSigner": true,
          "isOptional": true
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "priceLots",
          "type": "i64"
        },
        {
          "name": "maxBaseLots",
          "type": "i64"
        },
        {
          "name": "maxQuoteLotsIncludingFees",
          "type": "i64"
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "PlaceOrderType"
          }
        },
        {
          "name": "selfTradeBehavior",
          "type": {
            "defined": "SelfTradeBehavior"
          }
        },
        {
          "name": "limit",
          "type": "u8"
        }
      ],
      "returns": {
        "option": "u128"
      }
    },
    {
      "name": "consumeEvents",
      "docs": [
        "Process up to `limit` [events](crate::state::AnyEvent).",
        "",
        "When a user places a 'take' order, they do not know beforehand which",
        "market maker will have placed the 'make' order that they get executed",
        "against. This prevents them from passing in a market maker's",
        "[`OpenOrdersAccount`](crate::state::OpenOrdersAccount), which is needed",
        "to credit/debit the relevant tokens to/from the maker. As such, Openbook",
        "uses a 'crank' system, where `place_order` only emits events, and",
        "`consume_events` handles token settlement.",
        "",
        "Currently, there are two types of events: [`FillEvent`](crate::state::FillEvent)s",
        "and [`OutEvent`](crate::state::OutEvent)s.",
        "",
        "A `FillEvent` is emitted when an order is filled, and it is handled by",
        "debiting whatever the taker is selling from the taker and crediting",
        "it to the maker, and debiting whatever the taker is buying from the",
        "maker and crediting it to the taker. Note that *no tokens are moved*,",
        "these are just debits and credits to each party's [`Position`](crate::state::Position).",
        "",
        "An `OutEvent` is emitted when a limit order needs to be removed from",
        "the book during a `place_order` invocation, and it is handled by",
        "crediting whatever the maker would have sold (quote token in a bid,",
        "base token in an ask) back to the maker."
      ],
      "accounts": [
        {
          "name": "consumeEventsAdmin",
          "isMut": false,
          "isSigner": true,
          "isOptional": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u64"
        }
      ]
    },
    {
      "name": "cancelOrder",
      "docs": [
        "Cancel an order by its `order_id`.",
        "",
        "Note that this doesn't emit an [`OutEvent`](crate::state::OutEvent) because a",
        "maker knows that they will be passing in their own [`OpenOrdersAccount`](crate::state::OpenOrdersAccount)."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "cancelOrderByClientOrderId",
      "docs": [
        "Cancel an order by its `client_order_id`.",
        "",
        "Note that this doesn't emit an [`OutEvent`](crate::state::OutEvent) because a",
        "maker knows that they will be passing in their own [`OpenOrdersAccount`](crate::state::OpenOrdersAccount)."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "clientOrderId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "cancelAllOrders",
      "docs": [
        "Cancel up to `limit` orders."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u8"
        }
      ]
    },
    {
      "name": "cancelAllOrdersBySide",
      "docs": [
        "Cancel up to `limit` orders on a single side of the book."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "sideOption",
          "type": {
            "option": {
              "defined": "Side"
            }
          }
        },
        {
          "name": "limit",
          "type": "u8"
        }
      ]
    },
    {
      "name": "deposit",
      "docs": [
        "Desposit a certain amount of `base_amount_lots` and `quote_amount_lots`",
        "into one's [`Position`](crate::state::Position).",
        "",
        "Makers might wish to `deposit`, rather than have actual tokens moved for",
        "each trade, in order to reduce CUs."
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "tokenBaseAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenQuoteAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "baseAmountLots",
          "type": "u64"
        },
        {
          "name": "quoteAmountLots",
          "type": "u64"
        }
      ]
    },
    {
      "name": "settleFunds",
      "docs": [
        "Withdraw any available tokens."
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenBaseAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenQuoteAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "sweepFees",
      "docs": [
        "Sweep fees, as a [`Market`](crate::state::Market)'s admin."
      ],
      "accounts": [
        {
          "name": "collectFeeAdmin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenReceiverAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "closeMarket",
      "docs": [
        "Close a [`Market`](crate::state::Market)."
      ],
      "accounts": [
        {
          "name": "closeMarketAdmin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "solDestination",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "stubOracleCreate",
      "accounts": [
        {
          "name": "oracle",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "mint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "price",
          "type": {
            "defined": "I80F48"
          }
        }
      ]
    },
    {
      "name": "stubOracleClose",
      "accounts": [
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "oracle",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "solDestination",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "stubOracleSet",
      "accounts": [
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "oracle",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "price",
          "type": {
            "defined": "I80F48"
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "market",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "marketIndex",
            "docs": [
              "Index of this market"
            ],
            "type": "u32"
          },
          {
            "name": "bump",
            "docs": [
              "PDA bump"
            ],
            "type": "u8"
          },
          {
            "name": "baseDecimals",
            "docs": [
              "Number of decimals used for the base token.",
              "",
              "Used to convert the oracle's price into a native/native price."
            ],
            "type": "u8"
          },
          {
            "name": "quoteDecimals",
            "type": "u8"
          },
          {
            "name": "padding1",
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "collectFeeAdmin",
            "docs": [
              "Admin who can collect fees from the market"
            ],
            "type": "publicKey"
          },
          {
            "name": "openOrdersAdmin",
            "docs": [
              "Admin who must sign off on all order creations"
            ],
            "type": {
              "defined": "PodOption<Pubkey>"
            }
          },
          {
            "name": "consumeEventsAdmin",
            "docs": [
              "Admin who must sign off on all event consumptions"
            ],
            "type": {
              "defined": "PodOption<Pubkey>"
            }
          },
          {
            "name": "closeMarketAdmin",
            "docs": [
              "Admin who can close the market"
            ],
            "type": {
              "defined": "PodOption<Pubkey>"
            }
          },
          {
            "name": "name",
            "docs": [
              "Name. Trailing zero bytes are ignored."
            ],
            "type": {
              "array": [
                "u8",
                16
              ]
            }
          },
          {
            "name": "bids",
            "docs": [
              "Address of the BookSide account for bids"
            ],
            "type": "publicKey"
          },
          {
            "name": "asks",
            "docs": [
              "Address of the BookSide account for asks"
            ],
            "type": "publicKey"
          },
          {
            "name": "eventQueue",
            "docs": [
              "Address of the EventQueue account"
            ],
            "type": "publicKey"
          },
          {
            "name": "oracle",
            "docs": [
              "Oracle account address"
            ],
            "type": "publicKey"
          },
          {
            "name": "oracleConfig",
            "docs": [
              "Oracle configuration"
            ],
            "type": {
              "defined": "OracleConfig"
            }
          },
          {
            "name": "stablePriceModel",
            "docs": [
              "Maintains a stable price based on the oracle price that is less volatile."
            ],
            "type": {
              "defined": "StablePriceModel"
            }
          },
          {
            "name": "quoteLotSize",
            "docs": [
              "Number of quote native in a quote lot. Must be a power of 10.",
              "",
              "Primarily useful for increasing the tick size on the market: A lot price",
              "of 1 becomes a native price of quote_lot_size/base_lot_size becomes a",
              "ui price of quote_lot_size*base_decimals/base_lot_size/quote_decimals."
            ],
            "type": "i64"
          },
          {
            "name": "baseLotSize",
            "docs": [
              "Number of base native in a base lot. Must be a power of 10.",
              "",
              "Example: If base decimals for the underlying asset is 6, base lot size",
              "is 100 and and base position lots is 10_000 then base position native is",
              "1_000_000 and base position ui is 1."
            ],
            "type": "i64"
          },
          {
            "name": "seqNum",
            "docs": [
              "Total number of orders seen"
            ],
            "type": "u64"
          },
          {
            "name": "registrationTime",
            "docs": [
              "Timestamp in seconds that the market was registered at."
            ],
            "type": "u64"
          },
          {
            "name": "makerFee",
            "docs": [
              "Fees",
              "Fee when matching maker orders.",
              "maker_fee < 0 it means some of the taker_fees goes to the maker",
              "maker_fee > 0, it means no taker_fee to the maker, and maker fee goes to the referral"
            ],
            "type": {
              "defined": "I80F48"
            }
          },
          {
            "name": "takerFee",
            "docs": [
              "Fee for taker orders, always >= 0."
            ],
            "type": {
              "defined": "I80F48"
            }
          },
          {
            "name": "feePenalty",
            "docs": [
              "Fee (in quote native) to charge for ioc orders that don't match to avoid spam"
            ],
            "type": "u64"
          },
          {
            "name": "feesAccrued",
            "type": "i64"
          },
          {
            "name": "feesToReferrers",
            "type": "u64"
          },
          {
            "name": "vaultSignerNonce",
            "type": "u64"
          },
          {
            "name": "baseMint",
            "type": "publicKey"
          },
          {
            "name": "quoteMint",
            "type": "publicKey"
          },
          {
            "name": "baseVault",
            "type": "publicKey"
          },
          {
            "name": "baseDepositTotal",
            "type": "u64"
          },
          {
            "name": "baseFeesAccrued",
            "type": "u64"
          },
          {
            "name": "quoteVault",
            "type": "publicKey"
          },
          {
            "name": "quoteDepositTotal",
            "type": "u64"
          },
          {
            "name": "quoteFeesAccrued",
            "type": "u64"
          },
          {
            "name": "referrerRebatesAccrued",
            "type": "u64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                1768
              ]
            }
          }
        ]
      }
    },
    {
      "name": "openOrdersAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "name",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "delegate",
            "type": "publicKey"
          },
          {
            "name": "accountNum",
            "type": "u32"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "buybackFeesAccruedCurrent",
            "docs": [
              "Fees usable with the \"fees buyback\" feature.",
              "This tracks the ones that accrued in the current expiry interval."
            ],
            "type": "u64"
          },
          {
            "name": "buybackFeesAccruedPrevious",
            "docs": [
              "Fees buyback amount from the previous expiry interval."
            ],
            "type": "u64"
          },
          {
            "name": "buybackFeesExpiryTimestamp",
            "docs": [
              "End timestamp of the current expiry interval of the buyback fees amount."
            ],
            "type": "u64"
          },
          {
            "name": "position",
            "type": {
              "defined": "Position"
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                208
              ]
            }
          },
          {
            "name": "headerVersion",
            "type": "u8"
          },
          {
            "name": "padding3",
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "padding4",
            "type": "u32"
          },
          {
            "name": "openOrders",
            "type": {
              "vec": {
                "defined": "OpenOrder"
              }
            }
          }
        ]
      }
    },
    {
      "name": "stubOracle",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "group",
            "type": "publicKey"
          },
          {
            "name": "mint",
            "type": "publicKey"
          },
          {
            "name": "price",
            "type": {
              "defined": "I80F48"
            }
          },
          {
            "name": "lastUpdated",
            "type": "i64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                128
              ]
            }
          }
        ]
      }
    },
    {
      "name": "bookSide",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "roots",
            "type": {
              "array": [
                {
                  "defined": "OrderTreeRoot"
                },
                2
              ]
            }
          },
          {
            "name": "reservedRoots",
            "type": {
              "array": [
                {
                  "defined": "OrderTreeRoot"
                },
                4
              ]
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                256
              ]
            }
          },
          {
            "name": "nodes",
            "type": {
              "defined": "OrderTreeNodes"
            }
          }
        ]
      }
    },
    {
      "name": "eventQueue",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "header",
            "type": {
              "defined": "EventQueueHeader"
            }
          },
          {
            "name": "buf",
            "type": {
              "array": [
                {
                  "defined": "AnyEvent"
                },
                488
              ]
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                64
              ]
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "OpenOrdersAccountFixed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "name",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "delegate",
            "type": "publicKey"
          },
          {
            "name": "accountNum",
            "type": "u32"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "buybackFeesAccruedCurrent",
            "type": "u64"
          },
          {
            "name": "buybackFeesAccruedPrevious",
            "type": "u64"
          },
          {
            "name": "buybackFeesExpiryTimestamp",
            "type": "u64"
          },
          {
            "name": "position",
            "type": {
              "defined": "Position"
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                208
              ]
            }
          }
        ]
      }
    },
    {
      "name": "Position",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bidsBaseLots",
            "docs": [
              "Base lots in open bids"
            ],
            "type": "i64"
          },
          {
            "name": "asksBaseLots",
            "docs": [
              "Base lots in open asks"
            ],
            "type": "i64"
          },
          {
            "name": "baseFreeNative",
            "type": "u64"
          },
          {
            "name": "quoteFreeNative",
            "type": "u64"
          },
          {
            "name": "referrerRebatesAccrued",
            "type": "u64"
          },
          {
            "name": "makerVolume",
            "docs": [
              "Cumulative maker volume in quote native units",
              "",
              "(Display only)"
            ],
            "type": "u64"
          },
          {
            "name": "takerVolume",
            "docs": [
              "Cumulative taker volume in quote native units",
              "",
              "(Display only)"
            ],
            "type": "u64"
          },
          {
            "name": "avgEntryPricePerBaseLot",
            "docs": [
              "The native average entry price for the base lots of the current position.",
              "Reset to 0 when the base position reaches or crosses 0."
            ],
            "type": "f64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                88
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OpenOrder",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "sideAndTree",
            "type": "u8"
          },
          {
            "name": "padding1",
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "clientId",
            "type": "u64"
          },
          {
            "name": "pegLimit",
            "type": "i64"
          },
          {
            "name": "id",
            "type": "u128"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                64
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OracleConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "confFilter",
            "type": {
              "defined": "I80F48"
            }
          },
          {
            "name": "maxStalenessSlots",
            "type": "i64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                72
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OracleConfigParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "confFilter",
            "type": "f32"
          },
          {
            "name": "maxStalenessSlots",
            "type": {
              "option": "u32"
            }
          }
        ]
      }
    },
    {
      "name": "InnerNode",
      "docs": [
        "InnerNodes and LeafNodes compose the binary tree of orders.",
        "",
        "Each InnerNode has exactly two children, which are either InnerNodes themselves,",
        "or LeafNodes. The children share the top `prefix_len` bits of `key`. The left",
        "child has a 0 in the next bit, and the right a 1."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "tag",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "prefixLen",
            "docs": [
              "number of highest `key` bits that all children share",
              "e.g. if it's 2, the two highest bits of `key` will be the same on all children"
            ],
            "type": "u32"
          },
          {
            "name": "key",
            "docs": [
              "only the top `prefix_len` bits of `key` are relevant"
            ],
            "type": "u128"
          },
          {
            "name": "children",
            "docs": [
              "indexes into `BookSide::nodes`"
            ],
            "type": {
              "array": [
                "u32",
                2
              ]
            }
          },
          {
            "name": "childEarliestExpiry",
            "docs": [
              "The earliest expiry timestamp for the left and right subtrees.",
              "",
              "Needed to be able to find and remove expired orders without having to",
              "iterate through the whole bookside."
            ],
            "type": {
              "array": [
                "u64",
                2
              ]
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                72
              ]
            }
          }
        ]
      }
    },
    {
      "name": "LeafNode",
      "docs": [
        "LeafNodes represent an order in the binary tree"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "tag",
            "docs": [
              "NodeTag"
            ],
            "type": "u8"
          },
          {
            "name": "ownerSlot",
            "docs": [
              "Index into the owning OpenOrdersAccount's OpenOrders"
            ],
            "type": "u8"
          },
          {
            "name": "orderType",
            "docs": [
              "PostOrderType, this was added for TradingView move order"
            ],
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "timeInForce",
            "docs": [
              "Time in seconds after `timestamp` at which the order expires.",
              "A value of 0 means no expiry."
            ],
            "type": "u16"
          },
          {
            "name": "padding2",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "key",
            "docs": [
              "The binary tree key, see new_node_key()"
            ],
            "type": "u128"
          },
          {
            "name": "owner",
            "docs": [
              "Address of the owning OpenOrdersAccount"
            ],
            "type": "publicKey"
          },
          {
            "name": "quantity",
            "docs": [
              "Number of base lots to buy or sell, always >=1"
            ],
            "type": "i64"
          },
          {
            "name": "timestamp",
            "docs": [
              "The time the order was placed"
            ],
            "type": "u64"
          },
          {
            "name": "pegLimit",
            "docs": [
              "If the effective price of an oracle pegged order exceeds this limit,",
              "it will be considered invalid and may be removed.",
              "",
              "Only applicable in the oracle_pegged OrderTree"
            ],
            "type": "i64"
          },
          {
            "name": "clientOrderId",
            "docs": [
              "User defined id for this order, used in FillEvents"
            ],
            "type": "u64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          }
        ]
      }
    },
    {
      "name": "AnyNode",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "tag",
            "type": "u8"
          },
          {
            "name": "data",
            "type": {
              "array": [
                "u8",
                119
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OrderTreeRoot",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maybeNode",
            "type": "u32"
          },
          {
            "name": "leafCount",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "OrderTreeNodes",
      "docs": [
        "A binary tree on AnyNode::key()",
        "",
        "The key encodes the price in the top 64 bits."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "orderTreeType",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "bumpIndex",
            "type": "u32"
          },
          {
            "name": "freeListLen",
            "type": "u32"
          },
          {
            "name": "freeListHead",
            "type": "u32"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                512
              ]
            }
          },
          {
            "name": "nodes",
            "type": {
              "array": [
                {
                  "defined": "AnyNode"
                },
                1024
              ]
            }
          }
        ]
      }
    },
    {
      "name": "EventQueueHeader",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "head",
            "type": "u32"
          },
          {
            "name": "count",
            "type": "u32"
          },
          {
            "name": "seqNum",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "AnyEvent",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventType",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                199
              ]
            }
          }
        ]
      }
    },
    {
      "name": "FillEvent",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventType",
            "type": "u8"
          },
          {
            "name": "takerSide",
            "type": "u8"
          },
          {
            "name": "makerOut",
            "type": "u8"
          },
          {
            "name": "makerSlot",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                4
              ]
            }
          },
          {
            "name": "timestamp",
            "type": "u64"
          },
          {
            "name": "seqNum",
            "type": "u64"
          },
          {
            "name": "maker",
            "type": "publicKey"
          },
          {
            "name": "padding2",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "makerTimestamp",
            "type": "u64"
          },
          {
            "name": "taker",
            "type": "publicKey"
          },
          {
            "name": "padding3",
            "type": {
              "array": [
                "u8",
                16
              ]
            }
          },
          {
            "name": "takerClientOrderId",
            "type": "u64"
          },
          {
            "name": "padding4",
            "type": {
              "array": [
                "u8",
                16
              ]
            }
          },
          {
            "name": "price",
            "type": "i64"
          },
          {
            "name": "quantity",
            "type": "i64"
          },
          {
            "name": "makerClientOrderId",
            "type": "u64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                8
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OutEvent",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventType",
            "type": "u8"
          },
          {
            "name": "side",
            "type": "u8"
          },
          {
            "name": "ownerSlot",
            "type": "u8"
          },
          {
            "name": "padding0",
            "type": {
              "array": [
                "u8",
                5
              ]
            }
          },
          {
            "name": "timestamp",
            "type": "u64"
          },
          {
            "name": "seqNum",
            "type": "u64"
          },
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "quantity",
            "type": "i64"
          },
          {
            "name": "padding1",
            "type": {
              "array": [
                "u8",
                136
              ]
            }
          }
        ]
      }
    },
    {
      "name": "StablePriceModel",
      "docs": [
        "Maintains a \"stable_price\" based on the oracle price.",
        "",
        "The stable price follows the oracle price, but its relative rate of",
        "change is limited (to `stable_growth_limit`) and futher reduced if",
        "the oracle price is far from the `delay_price`.",
        "",
        "Conceptually the `delay_price` is itself a time delayed",
        "(`24 * delay_interval_seconds`, assume 24h) and relative rate of change limited",
        "function of the oracle price. It is implemented as averaging the oracle",
        "price over every `delay_interval_seconds` (assume 1h) and then applying the",
        "`delay_growth_limit` between intervals."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stablePrice",
            "docs": [
              "Current stable price to use in health"
            ],
            "type": "f64"
          },
          {
            "name": "lastUpdateTimestamp",
            "type": "u64"
          },
          {
            "name": "delayPrices",
            "docs": [
              "Stored delay_price for each delay_interval.",
              "If we want the delay_price to be 24h delayed, we would store one for each hour.",
              "This is used in a cyclical way: We use the maximally-delayed value at delay_interval_index",
              "and once enough time passes to move to the next delay interval, that gets overwritten and",
              "we use the next one."
            ],
            "type": {
              "array": [
                "f64",
                24
              ]
            }
          },
          {
            "name": "delayAccumulatorPrice",
            "docs": [
              "The delay price is based on an average over each delay_interval. The contributions",
              "to the average are summed up here."
            ],
            "type": "f64"
          },
          {
            "name": "delayAccumulatorTime",
            "docs": [
              "Accumulating the total time for the above average."
            ],
            "type": "u32"
          },
          {
            "name": "delayIntervalSeconds",
            "docs": [
              "Length of a delay_interval"
            ],
            "type": "u32"
          },
          {
            "name": "delayGrowthLimit",
            "docs": [
              "Maximal relative difference between two delay_price in consecutive intervals."
            ],
            "type": "f32"
          },
          {
            "name": "stableGrowthLimit",
            "docs": [
              "Maximal per-second relative difference of the stable price.",
              "It gets further reduced if stable and delay price disagree."
            ],
            "type": "f32"
          },
          {
            "name": "lastDelayIntervalIndex",
            "docs": [
              "The delay_interval_index that update() was last called on."
            ],
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                48
              ]
            }
          }
        ]
      }
    },
    {
      "name": "MarketIndex",
      "docs": [
        "Nothing in Rust shall use these types. They only exist so that the Anchor IDL",
        "knows about them and typescript can deserialize it."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "val",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "I80F48",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "val",
            "type": "i128"
          }
        ]
      }
    },
    {
      "name": "OracleType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Pyth"
          },
          {
            "name": "Stub"
          },
          {
            "name": "SwitchboardV1"
          },
          {
            "name": "SwitchboardV2"
          }
        ]
      }
    },
    {
      "name": "OrderState",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Valid"
          },
          {
            "name": "Invalid"
          },
          {
            "name": "Skipped"
          }
        ]
      }
    },
    {
      "name": "BookSideOrderTree",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fixed"
          },
          {
            "name": "OraclePegged"
          }
        ]
      }
    },
    {
      "name": "NodeTag",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Uninitialized"
          },
          {
            "name": "InnerNode"
          },
          {
            "name": "LeafNode"
          },
          {
            "name": "FreeNode"
          },
          {
            "name": "LastFreeNode"
          }
        ]
      }
    },
    {
      "name": "PlaceOrderType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Limit"
          },
          {
            "name": "ImmediateOrCancel"
          },
          {
            "name": "PostOnly"
          },
          {
            "name": "Market"
          },
          {
            "name": "PostOnlySlide"
          }
        ]
      }
    },
    {
      "name": "PostOrderType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Limit"
          },
          {
            "name": "PostOnly"
          },
          {
            "name": "PostOnlySlide"
          }
        ]
      }
    },
    {
      "name": "SelfTradeBehavior",
      "docs": [
        "Self trade behavior controls how taker orders interact with resting limit orders of the same account.",
        "This setting has no influence on placing a resting or oracle pegged limit order that does not match",
        "immediately, instead it's the responsibility of the user to correctly configure his taker orders."
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "DecrementTake"
          },
          {
            "name": "CancelProvide"
          },
          {
            "name": "AbortTransaction"
          }
        ]
      }
    },
    {
      "name": "Side",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Bid"
          },
          {
            "name": "Ask"
          }
        ]
      }
    },
    {
      "name": "SideAndOrderTree",
      "docs": [
        "SideAndOrderTree is a storage optimization, so we don't need two bytes for the data"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "BidFixed"
          },
          {
            "name": "AskFixed"
          },
          {
            "name": "BidOraclePegged"
          },
          {
            "name": "AskOraclePegged"
          }
        ]
      }
    },
    {
      "name": "OrderParams",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Market"
          },
          {
            "name": "ImmediateOrCancel",
            "fields": [
              {
                "name": "price_lots",
                "type": "i64"
              }
            ]
          },
          {
            "name": "Fixed",
            "fields": [
              {
                "name": "price_lots",
                "type": "i64"
              },
              {
                "name": "order_type",
                "type": {
                  "defined": "PostOrderType"
                }
              }
            ]
          },
          {
            "name": "OraclePegged",
            "fields": [
              {
                "name": "price_offset_lots",
                "type": "i64"
              },
              {
                "name": "order_type",
                "type": {
                  "defined": "PostOrderType"
                }
              },
              {
                "name": "peg_limit",
                "type": "i64"
              },
              {
                "name": "max_oracle_staleness_slots",
                "type": "i32"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "OrderTreeType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Bids"
          },
          {
            "name": "Asks"
          }
        ]
      }
    },
    {
      "name": "EventType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill"
          },
          {
            "name": "Out"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "BalanceLog",
      "fields": [
        {
          "name": "openOrdersAcc",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "basePosition",
          "type": "i64",
          "index": false
        },
        {
          "name": "quotePosition",
          "type": "i128",
          "index": false
        }
      ]
    },
    {
      "name": "DepositLog",
      "fields": [
        {
          "name": "openOrdersAcc",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "signer",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "quantity",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "FillLog",
      "fields": [
        {
          "name": "takerSide",
          "type": "u8",
          "index": false
        },
        {
          "name": "makerSlot",
          "type": "u8",
          "index": false
        },
        {
          "name": "makerOut",
          "type": "bool",
          "index": false
        },
        {
          "name": "timestamp",
          "type": "u64",
          "index": false
        },
        {
          "name": "seqNum",
          "type": "u64",
          "index": false
        },
        {
          "name": "maker",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "makerClientOrderId",
          "type": "u64",
          "index": false
        },
        {
          "name": "makerFee",
          "type": "f32",
          "index": false
        },
        {
          "name": "makerTimestamp",
          "type": "u64",
          "index": false
        },
        {
          "name": "taker",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "takerClientOrderId",
          "type": "u64",
          "index": false
        },
        {
          "name": "takerFee",
          "type": "f32",
          "index": false
        },
        {
          "name": "price",
          "type": "i64",
          "index": false
        },
        {
          "name": "quantity",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "MarketMetaDataLog",
      "fields": [
        {
          "name": "market",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "marketIndex",
          "type": "u32",
          "index": false
        },
        {
          "name": "baseDecimals",
          "type": "u8",
          "index": false
        },
        {
          "name": "quoteDecimals",
          "type": "u8",
          "index": false
        },
        {
          "name": "baseLotSize",
          "type": "i64",
          "index": false
        },
        {
          "name": "quoteLotSize",
          "type": "i64",
          "index": false
        },
        {
          "name": "oracle",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "TotalOrderFillEvent",
      "fields": [
        {
          "name": "side",
          "type": "u8",
          "index": false
        },
        {
          "name": "taker",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "totalQuantityPaid",
          "type": "u64",
          "index": false
        },
        {
          "name": "totalQuantityReceived",
          "type": "u64",
          "index": false
        },
        {
          "name": "fees",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "SomeError",
      "msg": ""
    },
    {
      "code": 6001,
      "name": "NotImplementedError",
      "msg": ""
    },
    {
      "code": 6002,
      "name": "MathError",
      "msg": "checked math error"
    },
    {
      "code": 6003,
      "name": "UnexpectedOracle",
      "msg": ""
    },
    {
      "code": 6004,
      "name": "UnknownOracleType",
      "msg": "oracle type cannot be determined"
    },
    {
      "code": 6005,
      "name": "InvalidBank",
      "msg": "invalid bank"
    },
    {
      "code": 6006,
      "name": "ProfitabilityMismatch",
      "msg": "account profitability is mismatched"
    },
    {
      "code": 6007,
      "name": "CannotSettleWithSelf",
      "msg": "cannot settle with self"
    },
    {
      "code": 6008,
      "name": "PositionDoesNotExist",
      "msg": "perp position does not exist"
    },
    {
      "code": 6009,
      "name": "MaxSettleAmountMustBeGreaterThanZero",
      "msg": "max settle amount must be greater than zero"
    },
    {
      "code": 6010,
      "name": "HasOpenOrders",
      "msg": "the perp position has open orders or unprocessed fill events"
    },
    {
      "code": 6011,
      "name": "OracleConfidence",
      "msg": "an oracle does not reach the confidence threshold"
    },
    {
      "code": 6012,
      "name": "OracleStale",
      "msg": "an oracle is stale"
    },
    {
      "code": 6013,
      "name": "SettlementAmountMustBePositive",
      "msg": "settlement amount must always be positive"
    },
    {
      "code": 6014,
      "name": "BankBorrowLimitReached",
      "msg": "bank utilization has reached limit"
    },
    {
      "code": 6015,
      "name": "BankNetBorrowsLimitReached",
      "msg": "bank net borrows has reached limit - this is an intermittent error - the limit will reset regularly"
    },
    {
      "code": 6016,
      "name": "TokenPositionDoesNotExist",
      "msg": "token position does not exist"
    },
    {
      "code": 6017,
      "name": "DepositsIntoLiquidatingMustRecover",
      "msg": "token deposits into accounts that are being liquidated must bring their health above the init threshold"
    },
    {
      "code": 6018,
      "name": "TokenInReduceOnlyMode",
      "msg": "token is in reduce only mode"
    },
    {
      "code": 6019,
      "name": "MarketInReduceOnlyMode",
      "msg": "market is in reduce only mode"
    },
    {
      "code": 6020,
      "name": "GroupIsHalted",
      "msg": "group is halted"
    },
    {
      "code": 6021,
      "name": "HasBaseLots",
      "msg": "the perp position has non-zero base lots"
    },
    {
      "code": 6022,
      "name": "HasOpenOrUnsettledSerum3Orders",
      "msg": "there are open or unsettled serum3 orders"
    },
    {
      "code": 6023,
      "name": "HasLiquidatableTokenPosition",
      "msg": "has liquidatable token position"
    },
    {
      "code": 6024,
      "name": "HasLiquidatableBasePosition",
      "msg": "has liquidatable perp base position"
    },
    {
      "code": 6025,
      "name": "HasLiquidatablePositivePnl",
      "msg": "has liquidatable positive perp pnl"
    },
    {
      "code": 6026,
      "name": "AccountIsFrozen",
      "msg": "account is frozen"
    },
    {
      "code": 6027,
      "name": "InitAssetWeightCantBeNegative",
      "msg": "Init Asset Weight can't be negative"
    },
    {
      "code": 6028,
      "name": "HasOpenTakerFills",
      "msg": "has open perp taker fills"
    },
    {
      "code": 6029,
      "name": "DepositLimit",
      "msg": "deposit crosses the current group deposit limit"
    },
    {
      "code": 6030,
      "name": "IxIsDisabled",
      "msg": "instruction is disabled"
    },
    {
      "code": 6031,
      "name": "NoLiquidatableBasePosition",
      "msg": "no liquidatable perp base position"
    },
    {
      "code": 6032,
      "name": "OrderIdNotFound",
      "msg": "perp order id not found on the orderbook"
    },
    {
      "code": 6033,
      "name": "HealthRegionBadInnerInstruction",
      "msg": "HealthRegions allow only specific instructions between Begin and End"
    },
    {
      "code": 6034,
      "name": "EventQueueContainsElements",
      "msg": "Event queue contains elements and market can't be closed"
    },
    {
      "code": 6035,
      "name": "InvalidFeesError",
      "msg": "Taker fees should be positive and if maker fees are negative, greater or equal to their abs value"
    },
    {
      "code": 6036,
      "name": "InvalidOrderType",
      "msg": "The order type is invalid. A taker order must be Market or ImmediateOrCancel"
    },
    {
      "code": 6037,
      "name": "InvalidFundsReceiver",
      "msg": "The receiver is invalid. Makes sure the receiver's owner is the market admin"
    },
    {
      "code": 6038,
      "name": "WouldSelfTrade",
      "msg": "would self trade"
    },
    {
      "code": 6039,
      "name": "NoCloseMarketAdmin",
      "msg": "This market does not have a `close_market_admin` and thus cannot be closed."
    },
    {
      "code": 6040,
      "name": "InvalidCloseMarketAdmin",
      "msg": "The signer of this transaction is not this market's `close_market_admin`."
    },
    {
      "code": 6041,
      "name": "MissingOpenOrdersAdmin",
      "msg": "This market requires `open_orders_admin` to sign all instructions that create orders."
    },
    {
      "code": 6042,
      "name": "InvalidOpenOrdersAdmin",
      "msg": "The `open_orders_admin` passed does not match this market's `open_orders_admin`."
    },
    {
      "code": 6043,
      "name": "MissingConsumeEventsAdmin",
      "msg": "This market requires `consume_events_admin` to sign all instructions that consume events."
    },
    {
      "code": 6044,
      "name": "InvalidConsumeEventsAdmin",
      "msg": "The `consume_events_admin` passed does not match this market's `consume_events_admin`."
    }
  ]
};

export const IDL: OpenbookV2 = {
  "version": "0.1.0",
  "name": "openbook_v2",
  "instructions": [
    {
      "name": "createMarket",
      "docs": [
        "Create a [`Market`](crate::state::Market) for a given token pair."
      ],
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Accounts are initialised by client,",
            "anchor discriminator is set first when ix exits,"
          ]
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "quoteMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "oracle",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "marketIndex",
          "type": "u32"
        },
        {
          "name": "name",
          "type": "string"
        },
        {
          "name": "oracleConfig",
          "type": {
            "defined": "OracleConfigParams"
          }
        },
        {
          "name": "quoteLotSize",
          "type": "i64"
        },
        {
          "name": "baseLotSize",
          "type": "i64"
        },
        {
          "name": "makerFee",
          "type": "f32"
        },
        {
          "name": "takerFee",
          "type": "f32"
        },
        {
          "name": "feePenalty",
          "type": "u64"
        },
        {
          "name": "collectFeeAdmin",
          "type": "publicKey"
        },
        {
          "name": "openOrdersAdmin",
          "type": {
            "option": "publicKey"
          }
        },
        {
          "name": "consumeEventsAdmin",
          "type": {
            "option": "publicKey"
          }
        },
        {
          "name": "closeMarketAdmin",
          "type": {
            "option": "publicKey"
          }
        }
      ]
    },
    {
      "name": "initOpenOrders",
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "accountNum",
          "type": "u32"
        },
        {
          "name": "openOrdersCount",
          "type": "u8"
        }
      ]
    },
    {
      "name": "placeOrder",
      "docs": [
        "Place an order.",
        "",
        "Different types of orders have different effects on the order book,",
        "as described in [`PlaceOrderType`](crate::state::PlaceOrderType).",
        "",
        "`price_lots` refers to the price in lots: the number of quote lots",
        "per base lot. It is ignored for `PlaceOrderType::Market` orders.",
        "",
        "`expiry_timestamp` is a unix timestamp for when this order should",
        "expire. If 0 is passed in, the order will never expire. If the time",
        "is in the past, the instruction is skipped. Timestamps in the future",
        "are reduced to now + 65,535s.",
        "",
        "`limit` determines the maximum number of orders from the book to fill,",
        "and can be used to limit CU spent. When the limit is reached, processing",
        "stops and the instruction succeeds."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "openOrdersAdmin",
          "isMut": false,
          "isSigner": true,
          "isOptional": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenDepositAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "oracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "priceLots",
          "type": "i64"
        },
        {
          "name": "maxBaseLots",
          "type": "i64"
        },
        {
          "name": "maxQuoteLotsIncludingFees",
          "type": "i64"
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "PlaceOrderType"
          }
        },
        {
          "name": "selfTradeBehavior",
          "type": {
            "defined": "SelfTradeBehavior"
          }
        },
        {
          "name": "expiryTimestamp",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u8"
        }
      ],
      "returns": {
        "option": "u128"
      }
    },
    {
      "name": "placeOrderPegged",
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "openOrdersAdmin",
          "isMut": false,
          "isSigner": true,
          "isOptional": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenDepositAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "oracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "priceOffsetLots",
          "type": "i64"
        },
        {
          "name": "pegLimit",
          "type": "i64"
        },
        {
          "name": "maxBaseLots",
          "type": "i64"
        },
        {
          "name": "maxQuoteLotsIncludingFees",
          "type": "i64"
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "PlaceOrderType"
          }
        },
        {
          "name": "selfTradeBehavior",
          "type": {
            "defined": "SelfTradeBehavior"
          }
        },
        {
          "name": "expiryTimestamp",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u8"
        },
        {
          "name": "maxOracleStalenessSlots",
          "type": "i32"
        }
      ],
      "returns": {
        "option": "u128"
      }
    },
    {
      "name": "placeTakeOrder",
      "docs": [
        "Place an order that shall take existing liquidity off of the book, not",
        "add a new order off the book.",
        "",
        "This type of order allows for instant token settlement for the taker."
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenDepositAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenReceiverAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "oracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "openOrdersAdmin",
          "isMut": false,
          "isSigner": true,
          "isOptional": true
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "priceLots",
          "type": "i64"
        },
        {
          "name": "maxBaseLots",
          "type": "i64"
        },
        {
          "name": "maxQuoteLotsIncludingFees",
          "type": "i64"
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "PlaceOrderType"
          }
        },
        {
          "name": "selfTradeBehavior",
          "type": {
            "defined": "SelfTradeBehavior"
          }
        },
        {
          "name": "limit",
          "type": "u8"
        }
      ],
      "returns": {
        "option": "u128"
      }
    },
    {
      "name": "consumeEvents",
      "docs": [
        "Process up to `limit` [events](crate::state::AnyEvent).",
        "",
        "When a user places a 'take' order, they do not know beforehand which",
        "market maker will have placed the 'make' order that they get executed",
        "against. This prevents them from passing in a market maker's",
        "[`OpenOrdersAccount`](crate::state::OpenOrdersAccount), which is needed",
        "to credit/debit the relevant tokens to/from the maker. As such, Openbook",
        "uses a 'crank' system, where `place_order` only emits events, and",
        "`consume_events` handles token settlement.",
        "",
        "Currently, there are two types of events: [`FillEvent`](crate::state::FillEvent)s",
        "and [`OutEvent`](crate::state::OutEvent)s.",
        "",
        "A `FillEvent` is emitted when an order is filled, and it is handled by",
        "debiting whatever the taker is selling from the taker and crediting",
        "it to the maker, and debiting whatever the taker is buying from the",
        "maker and crediting it to the taker. Note that *no tokens are moved*,",
        "these are just debits and credits to each party's [`Position`](crate::state::Position).",
        "",
        "An `OutEvent` is emitted when a limit order needs to be removed from",
        "the book during a `place_order` invocation, and it is handled by",
        "crediting whatever the maker would have sold (quote token in a bid,",
        "base token in an ask) back to the maker."
      ],
      "accounts": [
        {
          "name": "consumeEventsAdmin",
          "isMut": false,
          "isSigner": true,
          "isOptional": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u64"
        }
      ]
    },
    {
      "name": "cancelOrder",
      "docs": [
        "Cancel an order by its `order_id`.",
        "",
        "Note that this doesn't emit an [`OutEvent`](crate::state::OutEvent) because a",
        "maker knows that they will be passing in their own [`OpenOrdersAccount`](crate::state::OpenOrdersAccount)."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "cancelOrderByClientOrderId",
      "docs": [
        "Cancel an order by its `client_order_id`.",
        "",
        "Note that this doesn't emit an [`OutEvent`](crate::state::OutEvent) because a",
        "maker knows that they will be passing in their own [`OpenOrdersAccount`](crate::state::OpenOrdersAccount)."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "clientOrderId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "cancelAllOrders",
      "docs": [
        "Cancel up to `limit` orders."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u8"
        }
      ]
    },
    {
      "name": "cancelAllOrdersBySide",
      "docs": [
        "Cancel up to `limit` orders on a single side of the book."
      ],
      "accounts": [
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "sideOption",
          "type": {
            "option": {
              "defined": "Side"
            }
          }
        },
        {
          "name": "limit",
          "type": "u8"
        }
      ]
    },
    {
      "name": "deposit",
      "docs": [
        "Desposit a certain amount of `base_amount_lots` and `quote_amount_lots`",
        "into one's [`Position`](crate::state::Position).",
        "",
        "Makers might wish to `deposit`, rather than have actual tokens moved for",
        "each trade, in order to reduce CUs."
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "tokenBaseAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenQuoteAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "baseAmountLots",
          "type": "u64"
        },
        {
          "name": "quoteAmountLots",
          "type": "u64"
        }
      ]
    },
    {
      "name": "settleFunds",
      "docs": [
        "Withdraw any available tokens."
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenBaseAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenQuoteAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "sweepFees",
      "docs": [
        "Sweep fees, as a [`Market`](crate::state::Market)'s admin."
      ],
      "accounts": [
        {
          "name": "collectFeeAdmin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenReceiverAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "closeMarket",
      "docs": [
        "Close a [`Market`](crate::state::Market)."
      ],
      "accounts": [
        {
          "name": "closeMarketAdmin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "solDestination",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "stubOracleCreate",
      "accounts": [
        {
          "name": "oracle",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "mint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "price",
          "type": {
            "defined": "I80F48"
          }
        }
      ]
    },
    {
      "name": "stubOracleClose",
      "accounts": [
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "oracle",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "solDestination",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "stubOracleSet",
      "accounts": [
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "oracle",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "price",
          "type": {
            "defined": "I80F48"
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "market",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "marketIndex",
            "docs": [
              "Index of this market"
            ],
            "type": "u32"
          },
          {
            "name": "bump",
            "docs": [
              "PDA bump"
            ],
            "type": "u8"
          },
          {
            "name": "baseDecimals",
            "docs": [
              "Number of decimals used for the base token.",
              "",
              "Used to convert the oracle's price into a native/native price."
            ],
            "type": "u8"
          },
          {
            "name": "quoteDecimals",
            "type": "u8"
          },
          {
            "name": "padding1",
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "collectFeeAdmin",
            "docs": [
              "Admin who can collect fees from the market"
            ],
            "type": "publicKey"
          },
          {
            "name": "openOrdersAdmin",
            "docs": [
              "Admin who must sign off on all order creations"
            ],
            "type": {
              "defined": "PodOption<Pubkey>"
            }
          },
          {
            "name": "consumeEventsAdmin",
            "docs": [
              "Admin who must sign off on all event consumptions"
            ],
            "type": {
              "defined": "PodOption<Pubkey>"
            }
          },
          {
            "name": "closeMarketAdmin",
            "docs": [
              "Admin who can close the market"
            ],
            "type": {
              "defined": "PodOption<Pubkey>"
            }
          },
          {
            "name": "name",
            "docs": [
              "Name. Trailing zero bytes are ignored."
            ],
            "type": {
              "array": [
                "u8",
                16
              ]
            }
          },
          {
            "name": "bids",
            "docs": [
              "Address of the BookSide account for bids"
            ],
            "type": "publicKey"
          },
          {
            "name": "asks",
            "docs": [
              "Address of the BookSide account for asks"
            ],
            "type": "publicKey"
          },
          {
            "name": "eventQueue",
            "docs": [
              "Address of the EventQueue account"
            ],
            "type": "publicKey"
          },
          {
            "name": "oracle",
            "docs": [
              "Oracle account address"
            ],
            "type": "publicKey"
          },
          {
            "name": "oracleConfig",
            "docs": [
              "Oracle configuration"
            ],
            "type": {
              "defined": "OracleConfig"
            }
          },
          {
            "name": "stablePriceModel",
            "docs": [
              "Maintains a stable price based on the oracle price that is less volatile."
            ],
            "type": {
              "defined": "StablePriceModel"
            }
          },
          {
            "name": "quoteLotSize",
            "docs": [
              "Number of quote native in a quote lot. Must be a power of 10.",
              "",
              "Primarily useful for increasing the tick size on the market: A lot price",
              "of 1 becomes a native price of quote_lot_size/base_lot_size becomes a",
              "ui price of quote_lot_size*base_decimals/base_lot_size/quote_decimals."
            ],
            "type": "i64"
          },
          {
            "name": "baseLotSize",
            "docs": [
              "Number of base native in a base lot. Must be a power of 10.",
              "",
              "Example: If base decimals for the underlying asset is 6, base lot size",
              "is 100 and and base position lots is 10_000 then base position native is",
              "1_000_000 and base position ui is 1."
            ],
            "type": "i64"
          },
          {
            "name": "seqNum",
            "docs": [
              "Total number of orders seen"
            ],
            "type": "u64"
          },
          {
            "name": "registrationTime",
            "docs": [
              "Timestamp in seconds that the market was registered at."
            ],
            "type": "u64"
          },
          {
            "name": "makerFee",
            "docs": [
              "Fees",
              "Fee when matching maker orders.",
              "maker_fee < 0 it means some of the taker_fees goes to the maker",
              "maker_fee > 0, it means no taker_fee to the maker, and maker fee goes to the referral"
            ],
            "type": {
              "defined": "I80F48"
            }
          },
          {
            "name": "takerFee",
            "docs": [
              "Fee for taker orders, always >= 0."
            ],
            "type": {
              "defined": "I80F48"
            }
          },
          {
            "name": "feePenalty",
            "docs": [
              "Fee (in quote native) to charge for ioc orders that don't match to avoid spam"
            ],
            "type": "u64"
          },
          {
            "name": "feesAccrued",
            "type": "i64"
          },
          {
            "name": "feesToReferrers",
            "type": "u64"
          },
          {
            "name": "vaultSignerNonce",
            "type": "u64"
          },
          {
            "name": "baseMint",
            "type": "publicKey"
          },
          {
            "name": "quoteMint",
            "type": "publicKey"
          },
          {
            "name": "baseVault",
            "type": "publicKey"
          },
          {
            "name": "baseDepositTotal",
            "type": "u64"
          },
          {
            "name": "baseFeesAccrued",
            "type": "u64"
          },
          {
            "name": "quoteVault",
            "type": "publicKey"
          },
          {
            "name": "quoteDepositTotal",
            "type": "u64"
          },
          {
            "name": "quoteFeesAccrued",
            "type": "u64"
          },
          {
            "name": "referrerRebatesAccrued",
            "type": "u64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                1768
              ]
            }
          }
        ]
      }
    },
    {
      "name": "openOrdersAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "name",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "delegate",
            "type": "publicKey"
          },
          {
            "name": "accountNum",
            "type": "u32"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "buybackFeesAccruedCurrent",
            "docs": [
              "Fees usable with the \"fees buyback\" feature.",
              "This tracks the ones that accrued in the current expiry interval."
            ],
            "type": "u64"
          },
          {
            "name": "buybackFeesAccruedPrevious",
            "docs": [
              "Fees buyback amount from the previous expiry interval."
            ],
            "type": "u64"
          },
          {
            "name": "buybackFeesExpiryTimestamp",
            "docs": [
              "End timestamp of the current expiry interval of the buyback fees amount."
            ],
            "type": "u64"
          },
          {
            "name": "position",
            "type": {
              "defined": "Position"
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                208
              ]
            }
          },
          {
            "name": "headerVersion",
            "type": "u8"
          },
          {
            "name": "padding3",
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "padding4",
            "type": "u32"
          },
          {
            "name": "openOrders",
            "type": {
              "vec": {
                "defined": "OpenOrder"
              }
            }
          }
        ]
      }
    },
    {
      "name": "stubOracle",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "group",
            "type": "publicKey"
          },
          {
            "name": "mint",
            "type": "publicKey"
          },
          {
            "name": "price",
            "type": {
              "defined": "I80F48"
            }
          },
          {
            "name": "lastUpdated",
            "type": "i64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                128
              ]
            }
          }
        ]
      }
    },
    {
      "name": "bookSide",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "roots",
            "type": {
              "array": [
                {
                  "defined": "OrderTreeRoot"
                },
                2
              ]
            }
          },
          {
            "name": "reservedRoots",
            "type": {
              "array": [
                {
                  "defined": "OrderTreeRoot"
                },
                4
              ]
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                256
              ]
            }
          },
          {
            "name": "nodes",
            "type": {
              "defined": "OrderTreeNodes"
            }
          }
        ]
      }
    },
    {
      "name": "eventQueue",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "header",
            "type": {
              "defined": "EventQueueHeader"
            }
          },
          {
            "name": "buf",
            "type": {
              "array": [
                {
                  "defined": "AnyEvent"
                },
                488
              ]
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                64
              ]
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "OpenOrdersAccountFixed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "name",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "delegate",
            "type": "publicKey"
          },
          {
            "name": "accountNum",
            "type": "u32"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "buybackFeesAccruedCurrent",
            "type": "u64"
          },
          {
            "name": "buybackFeesAccruedPrevious",
            "type": "u64"
          },
          {
            "name": "buybackFeesExpiryTimestamp",
            "type": "u64"
          },
          {
            "name": "position",
            "type": {
              "defined": "Position"
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                208
              ]
            }
          }
        ]
      }
    },
    {
      "name": "Position",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bidsBaseLots",
            "docs": [
              "Base lots in open bids"
            ],
            "type": "i64"
          },
          {
            "name": "asksBaseLots",
            "docs": [
              "Base lots in open asks"
            ],
            "type": "i64"
          },
          {
            "name": "baseFreeNative",
            "type": "u64"
          },
          {
            "name": "quoteFreeNative",
            "type": "u64"
          },
          {
            "name": "referrerRebatesAccrued",
            "type": "u64"
          },
          {
            "name": "makerVolume",
            "docs": [
              "Cumulative maker volume in quote native units",
              "",
              "(Display only)"
            ],
            "type": "u64"
          },
          {
            "name": "takerVolume",
            "docs": [
              "Cumulative taker volume in quote native units",
              "",
              "(Display only)"
            ],
            "type": "u64"
          },
          {
            "name": "avgEntryPricePerBaseLot",
            "docs": [
              "The native average entry price for the base lots of the current position.",
              "Reset to 0 when the base position reaches or crosses 0."
            ],
            "type": "f64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                88
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OpenOrder",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "sideAndTree",
            "type": "u8"
          },
          {
            "name": "padding1",
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "clientId",
            "type": "u64"
          },
          {
            "name": "pegLimit",
            "type": "i64"
          },
          {
            "name": "id",
            "type": "u128"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                64
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OracleConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "confFilter",
            "type": {
              "defined": "I80F48"
            }
          },
          {
            "name": "maxStalenessSlots",
            "type": "i64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                72
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OracleConfigParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "confFilter",
            "type": "f32"
          },
          {
            "name": "maxStalenessSlots",
            "type": {
              "option": "u32"
            }
          }
        ]
      }
    },
    {
      "name": "InnerNode",
      "docs": [
        "InnerNodes and LeafNodes compose the binary tree of orders.",
        "",
        "Each InnerNode has exactly two children, which are either InnerNodes themselves,",
        "or LeafNodes. The children share the top `prefix_len` bits of `key`. The left",
        "child has a 0 in the next bit, and the right a 1."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "tag",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "prefixLen",
            "docs": [
              "number of highest `key` bits that all children share",
              "e.g. if it's 2, the two highest bits of `key` will be the same on all children"
            ],
            "type": "u32"
          },
          {
            "name": "key",
            "docs": [
              "only the top `prefix_len` bits of `key` are relevant"
            ],
            "type": "u128"
          },
          {
            "name": "children",
            "docs": [
              "indexes into `BookSide::nodes`"
            ],
            "type": {
              "array": [
                "u32",
                2
              ]
            }
          },
          {
            "name": "childEarliestExpiry",
            "docs": [
              "The earliest expiry timestamp for the left and right subtrees.",
              "",
              "Needed to be able to find and remove expired orders without having to",
              "iterate through the whole bookside."
            ],
            "type": {
              "array": [
                "u64",
                2
              ]
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                72
              ]
            }
          }
        ]
      }
    },
    {
      "name": "LeafNode",
      "docs": [
        "LeafNodes represent an order in the binary tree"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "tag",
            "docs": [
              "NodeTag"
            ],
            "type": "u8"
          },
          {
            "name": "ownerSlot",
            "docs": [
              "Index into the owning OpenOrdersAccount's OpenOrders"
            ],
            "type": "u8"
          },
          {
            "name": "orderType",
            "docs": [
              "PostOrderType, this was added for TradingView move order"
            ],
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "timeInForce",
            "docs": [
              "Time in seconds after `timestamp` at which the order expires.",
              "A value of 0 means no expiry."
            ],
            "type": "u16"
          },
          {
            "name": "padding2",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "key",
            "docs": [
              "The binary tree key, see new_node_key()"
            ],
            "type": "u128"
          },
          {
            "name": "owner",
            "docs": [
              "Address of the owning OpenOrdersAccount"
            ],
            "type": "publicKey"
          },
          {
            "name": "quantity",
            "docs": [
              "Number of base lots to buy or sell, always >=1"
            ],
            "type": "i64"
          },
          {
            "name": "timestamp",
            "docs": [
              "The time the order was placed"
            ],
            "type": "u64"
          },
          {
            "name": "pegLimit",
            "docs": [
              "If the effective price of an oracle pegged order exceeds this limit,",
              "it will be considered invalid and may be removed.",
              "",
              "Only applicable in the oracle_pegged OrderTree"
            ],
            "type": "i64"
          },
          {
            "name": "clientOrderId",
            "docs": [
              "User defined id for this order, used in FillEvents"
            ],
            "type": "u64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          }
        ]
      }
    },
    {
      "name": "AnyNode",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "tag",
            "type": "u8"
          },
          {
            "name": "data",
            "type": {
              "array": [
                "u8",
                119
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OrderTreeRoot",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maybeNode",
            "type": "u32"
          },
          {
            "name": "leafCount",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "OrderTreeNodes",
      "docs": [
        "A binary tree on AnyNode::key()",
        "",
        "The key encodes the price in the top 64 bits."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "orderTreeType",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "bumpIndex",
            "type": "u32"
          },
          {
            "name": "freeListLen",
            "type": "u32"
          },
          {
            "name": "freeListHead",
            "type": "u32"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                512
              ]
            }
          },
          {
            "name": "nodes",
            "type": {
              "array": [
                {
                  "defined": "AnyNode"
                },
                1024
              ]
            }
          }
        ]
      }
    },
    {
      "name": "EventQueueHeader",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "head",
            "type": "u32"
          },
          {
            "name": "count",
            "type": "u32"
          },
          {
            "name": "seqNum",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "AnyEvent",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventType",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                199
              ]
            }
          }
        ]
      }
    },
    {
      "name": "FillEvent",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventType",
            "type": "u8"
          },
          {
            "name": "takerSide",
            "type": "u8"
          },
          {
            "name": "makerOut",
            "type": "u8"
          },
          {
            "name": "makerSlot",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                4
              ]
            }
          },
          {
            "name": "timestamp",
            "type": "u64"
          },
          {
            "name": "seqNum",
            "type": "u64"
          },
          {
            "name": "maker",
            "type": "publicKey"
          },
          {
            "name": "padding2",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "makerTimestamp",
            "type": "u64"
          },
          {
            "name": "taker",
            "type": "publicKey"
          },
          {
            "name": "padding3",
            "type": {
              "array": [
                "u8",
                16
              ]
            }
          },
          {
            "name": "takerClientOrderId",
            "type": "u64"
          },
          {
            "name": "padding4",
            "type": {
              "array": [
                "u8",
                16
              ]
            }
          },
          {
            "name": "price",
            "type": "i64"
          },
          {
            "name": "quantity",
            "type": "i64"
          },
          {
            "name": "makerClientOrderId",
            "type": "u64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                8
              ]
            }
          }
        ]
      }
    },
    {
      "name": "OutEvent",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventType",
            "type": "u8"
          },
          {
            "name": "side",
            "type": "u8"
          },
          {
            "name": "ownerSlot",
            "type": "u8"
          },
          {
            "name": "padding0",
            "type": {
              "array": [
                "u8",
                5
              ]
            }
          },
          {
            "name": "timestamp",
            "type": "u64"
          },
          {
            "name": "seqNum",
            "type": "u64"
          },
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "quantity",
            "type": "i64"
          },
          {
            "name": "padding1",
            "type": {
              "array": [
                "u8",
                136
              ]
            }
          }
        ]
      }
    },
    {
      "name": "StablePriceModel",
      "docs": [
        "Maintains a \"stable_price\" based on the oracle price.",
        "",
        "The stable price follows the oracle price, but its relative rate of",
        "change is limited (to `stable_growth_limit`) and futher reduced if",
        "the oracle price is far from the `delay_price`.",
        "",
        "Conceptually the `delay_price` is itself a time delayed",
        "(`24 * delay_interval_seconds`, assume 24h) and relative rate of change limited",
        "function of the oracle price. It is implemented as averaging the oracle",
        "price over every `delay_interval_seconds` (assume 1h) and then applying the",
        "`delay_growth_limit` between intervals."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stablePrice",
            "docs": [
              "Current stable price to use in health"
            ],
            "type": "f64"
          },
          {
            "name": "lastUpdateTimestamp",
            "type": "u64"
          },
          {
            "name": "delayPrices",
            "docs": [
              "Stored delay_price for each delay_interval.",
              "If we want the delay_price to be 24h delayed, we would store one for each hour.",
              "This is used in a cyclical way: We use the maximally-delayed value at delay_interval_index",
              "and once enough time passes to move to the next delay interval, that gets overwritten and",
              "we use the next one."
            ],
            "type": {
              "array": [
                "f64",
                24
              ]
            }
          },
          {
            "name": "delayAccumulatorPrice",
            "docs": [
              "The delay price is based on an average over each delay_interval. The contributions",
              "to the average are summed up here."
            ],
            "type": "f64"
          },
          {
            "name": "delayAccumulatorTime",
            "docs": [
              "Accumulating the total time for the above average."
            ],
            "type": "u32"
          },
          {
            "name": "delayIntervalSeconds",
            "docs": [
              "Length of a delay_interval"
            ],
            "type": "u32"
          },
          {
            "name": "delayGrowthLimit",
            "docs": [
              "Maximal relative difference between two delay_price in consecutive intervals."
            ],
            "type": "f32"
          },
          {
            "name": "stableGrowthLimit",
            "docs": [
              "Maximal per-second relative difference of the stable price.",
              "It gets further reduced if stable and delay price disagree."
            ],
            "type": "f32"
          },
          {
            "name": "lastDelayIntervalIndex",
            "docs": [
              "The delay_interval_index that update() was last called on."
            ],
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                48
              ]
            }
          }
        ]
      }
    },
    {
      "name": "MarketIndex",
      "docs": [
        "Nothing in Rust shall use these types. They only exist so that the Anchor IDL",
        "knows about them and typescript can deserialize it."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "val",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "I80F48",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "val",
            "type": "i128"
          }
        ]
      }
    },
    {
      "name": "OracleType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Pyth"
          },
          {
            "name": "Stub"
          },
          {
            "name": "SwitchboardV1"
          },
          {
            "name": "SwitchboardV2"
          }
        ]
      }
    },
    {
      "name": "OrderState",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Valid"
          },
          {
            "name": "Invalid"
          },
          {
            "name": "Skipped"
          }
        ]
      }
    },
    {
      "name": "BookSideOrderTree",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fixed"
          },
          {
            "name": "OraclePegged"
          }
        ]
      }
    },
    {
      "name": "NodeTag",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Uninitialized"
          },
          {
            "name": "InnerNode"
          },
          {
            "name": "LeafNode"
          },
          {
            "name": "FreeNode"
          },
          {
            "name": "LastFreeNode"
          }
        ]
      }
    },
    {
      "name": "PlaceOrderType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Limit"
          },
          {
            "name": "ImmediateOrCancel"
          },
          {
            "name": "PostOnly"
          },
          {
            "name": "Market"
          },
          {
            "name": "PostOnlySlide"
          }
        ]
      }
    },
    {
      "name": "PostOrderType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Limit"
          },
          {
            "name": "PostOnly"
          },
          {
            "name": "PostOnlySlide"
          }
        ]
      }
    },
    {
      "name": "SelfTradeBehavior",
      "docs": [
        "Self trade behavior controls how taker orders interact with resting limit orders of the same account.",
        "This setting has no influence on placing a resting or oracle pegged limit order that does not match",
        "immediately, instead it's the responsibility of the user to correctly configure his taker orders."
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "DecrementTake"
          },
          {
            "name": "CancelProvide"
          },
          {
            "name": "AbortTransaction"
          }
        ]
      }
    },
    {
      "name": "Side",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Bid"
          },
          {
            "name": "Ask"
          }
        ]
      }
    },
    {
      "name": "SideAndOrderTree",
      "docs": [
        "SideAndOrderTree is a storage optimization, so we don't need two bytes for the data"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "BidFixed"
          },
          {
            "name": "AskFixed"
          },
          {
            "name": "BidOraclePegged"
          },
          {
            "name": "AskOraclePegged"
          }
        ]
      }
    },
    {
      "name": "OrderParams",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Market"
          },
          {
            "name": "ImmediateOrCancel",
            "fields": [
              {
                "name": "price_lots",
                "type": "i64"
              }
            ]
          },
          {
            "name": "Fixed",
            "fields": [
              {
                "name": "price_lots",
                "type": "i64"
              },
              {
                "name": "order_type",
                "type": {
                  "defined": "PostOrderType"
                }
              }
            ]
          },
          {
            "name": "OraclePegged",
            "fields": [
              {
                "name": "price_offset_lots",
                "type": "i64"
              },
              {
                "name": "order_type",
                "type": {
                  "defined": "PostOrderType"
                }
              },
              {
                "name": "peg_limit",
                "type": "i64"
              },
              {
                "name": "max_oracle_staleness_slots",
                "type": "i32"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "OrderTreeType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Bids"
          },
          {
            "name": "Asks"
          }
        ]
      }
    },
    {
      "name": "EventType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill"
          },
          {
            "name": "Out"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "BalanceLog",
      "fields": [
        {
          "name": "openOrdersAcc",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "basePosition",
          "type": "i64",
          "index": false
        },
        {
          "name": "quotePosition",
          "type": "i128",
          "index": false
        }
      ]
    },
    {
      "name": "DepositLog",
      "fields": [
        {
          "name": "openOrdersAcc",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "signer",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "quantity",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "FillLog",
      "fields": [
        {
          "name": "takerSide",
          "type": "u8",
          "index": false
        },
        {
          "name": "makerSlot",
          "type": "u8",
          "index": false
        },
        {
          "name": "makerOut",
          "type": "bool",
          "index": false
        },
        {
          "name": "timestamp",
          "type": "u64",
          "index": false
        },
        {
          "name": "seqNum",
          "type": "u64",
          "index": false
        },
        {
          "name": "maker",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "makerClientOrderId",
          "type": "u64",
          "index": false
        },
        {
          "name": "makerFee",
          "type": "f32",
          "index": false
        },
        {
          "name": "makerTimestamp",
          "type": "u64",
          "index": false
        },
        {
          "name": "taker",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "takerClientOrderId",
          "type": "u64",
          "index": false
        },
        {
          "name": "takerFee",
          "type": "f32",
          "index": false
        },
        {
          "name": "price",
          "type": "i64",
          "index": false
        },
        {
          "name": "quantity",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "MarketMetaDataLog",
      "fields": [
        {
          "name": "market",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "marketIndex",
          "type": "u32",
          "index": false
        },
        {
          "name": "baseDecimals",
          "type": "u8",
          "index": false
        },
        {
          "name": "quoteDecimals",
          "type": "u8",
          "index": false
        },
        {
          "name": "baseLotSize",
          "type": "i64",
          "index": false
        },
        {
          "name": "quoteLotSize",
          "type": "i64",
          "index": false
        },
        {
          "name": "oracle",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "TotalOrderFillEvent",
      "fields": [
        {
          "name": "side",
          "type": "u8",
          "index": false
        },
        {
          "name": "taker",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "totalQuantityPaid",
          "type": "u64",
          "index": false
        },
        {
          "name": "totalQuantityReceived",
          "type": "u64",
          "index": false
        },
        {
          "name": "fees",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "SomeError",
      "msg": ""
    },
    {
      "code": 6001,
      "name": "NotImplementedError",
      "msg": ""
    },
    {
      "code": 6002,
      "name": "MathError",
      "msg": "checked math error"
    },
    {
      "code": 6003,
      "name": "UnexpectedOracle",
      "msg": ""
    },
    {
      "code": 6004,
      "name": "UnknownOracleType",
      "msg": "oracle type cannot be determined"
    },
    {
      "code": 6005,
      "name": "InvalidBank",
      "msg": "invalid bank"
    },
    {
      "code": 6006,
      "name": "ProfitabilityMismatch",
      "msg": "account profitability is mismatched"
    },
    {
      "code": 6007,
      "name": "CannotSettleWithSelf",
      "msg": "cannot settle with self"
    },
    {
      "code": 6008,
      "name": "PositionDoesNotExist",
      "msg": "perp position does not exist"
    },
    {
      "code": 6009,
      "name": "MaxSettleAmountMustBeGreaterThanZero",
      "msg": "max settle amount must be greater than zero"
    },
    {
      "code": 6010,
      "name": "HasOpenOrders",
      "msg": "the perp position has open orders or unprocessed fill events"
    },
    {
      "code": 6011,
      "name": "OracleConfidence",
      "msg": "an oracle does not reach the confidence threshold"
    },
    {
      "code": 6012,
      "name": "OracleStale",
      "msg": "an oracle is stale"
    },
    {
      "code": 6013,
      "name": "SettlementAmountMustBePositive",
      "msg": "settlement amount must always be positive"
    },
    {
      "code": 6014,
      "name": "BankBorrowLimitReached",
      "msg": "bank utilization has reached limit"
    },
    {
      "code": 6015,
      "name": "BankNetBorrowsLimitReached",
      "msg": "bank net borrows has reached limit - this is an intermittent error - the limit will reset regularly"
    },
    {
      "code": 6016,
      "name": "TokenPositionDoesNotExist",
      "msg": "token position does not exist"
    },
    {
      "code": 6017,
      "name": "DepositsIntoLiquidatingMustRecover",
      "msg": "token deposits into accounts that are being liquidated must bring their health above the init threshold"
    },
    {
      "code": 6018,
      "name": "TokenInReduceOnlyMode",
      "msg": "token is in reduce only mode"
    },
    {
      "code": 6019,
      "name": "MarketInReduceOnlyMode",
      "msg": "market is in reduce only mode"
    },
    {
      "code": 6020,
      "name": "GroupIsHalted",
      "msg": "group is halted"
    },
    {
      "code": 6021,
      "name": "HasBaseLots",
      "msg": "the perp position has non-zero base lots"
    },
    {
      "code": 6022,
      "name": "HasOpenOrUnsettledSerum3Orders",
      "msg": "there are open or unsettled serum3 orders"
    },
    {
      "code": 6023,
      "name": "HasLiquidatableTokenPosition",
      "msg": "has liquidatable token position"
    },
    {
      "code": 6024,
      "name": "HasLiquidatableBasePosition",
      "msg": "has liquidatable perp base position"
    },
    {
      "code": 6025,
      "name": "HasLiquidatablePositivePnl",
      "msg": "has liquidatable positive perp pnl"
    },
    {
      "code": 6026,
      "name": "AccountIsFrozen",
      "msg": "account is frozen"
    },
    {
      "code": 6027,
      "name": "InitAssetWeightCantBeNegative",
      "msg": "Init Asset Weight can't be negative"
    },
    {
      "code": 6028,
      "name": "HasOpenTakerFills",
      "msg": "has open perp taker fills"
    },
    {
      "code": 6029,
      "name": "DepositLimit",
      "msg": "deposit crosses the current group deposit limit"
    },
    {
      "code": 6030,
      "name": "IxIsDisabled",
      "msg": "instruction is disabled"
    },
    {
      "code": 6031,
      "name": "NoLiquidatableBasePosition",
      "msg": "no liquidatable perp base position"
    },
    {
      "code": 6032,
      "name": "OrderIdNotFound",
      "msg": "perp order id not found on the orderbook"
    },
    {
      "code": 6033,
      "name": "HealthRegionBadInnerInstruction",
      "msg": "HealthRegions allow only specific instructions between Begin and End"
    },
    {
      "code": 6034,
      "name": "EventQueueContainsElements",
      "msg": "Event queue contains elements and market can't be closed"
    },
    {
      "code": 6035,
      "name": "InvalidFeesError",
      "msg": "Taker fees should be positive and if maker fees are negative, greater or equal to their abs value"
    },
    {
      "code": 6036,
      "name": "InvalidOrderType",
      "msg": "The order type is invalid. A taker order must be Market or ImmediateOrCancel"
    },
    {
      "code": 6037,
      "name": "InvalidFundsReceiver",
      "msg": "The receiver is invalid. Makes sure the receiver's owner is the market admin"
    },
    {
      "code": 6038,
      "name": "WouldSelfTrade",
      "msg": "would self trade"
    },
    {
      "code": 6039,
      "name": "NoCloseMarketAdmin",
      "msg": "This market does not have a `close_market_admin` and thus cannot be closed."
    },
    {
      "code": 6040,
      "name": "InvalidCloseMarketAdmin",
      "msg": "The signer of this transaction is not this market's `close_market_admin`."
    },
    {
      "code": 6041,
      "name": "MissingOpenOrdersAdmin",
      "msg": "This market requires `open_orders_admin` to sign all instructions that create orders."
    },
    {
      "code": 6042,
      "name": "InvalidOpenOrdersAdmin",
      "msg": "The `open_orders_admin` passed does not match this market's `open_orders_admin`."
    },
    {
      "code": 6043,
      "name": "MissingConsumeEventsAdmin",
      "msg": "This market requires `consume_events_admin` to sign all instructions that consume events."
    },
    {
      "code": 6044,
      "name": "InvalidConsumeEventsAdmin",
      "msg": "The `consume_events_admin` passed does not match this market's `consume_events_admin`."
    }
  ]
};
