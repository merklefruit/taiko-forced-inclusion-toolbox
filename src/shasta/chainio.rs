use alloy::sol;

sol! {
    /// @title LibBlobs
    /// @notice Library for handling blobs.
    /// @custom:security-contact security@taiko.xyz
    #[derive(Debug)]
    library LibBlobs {
        // ---------------------------------------------------------------
        // Constants
        // ---------------------------------------------------------------
        uint256 internal constant FIELD_ELEMENT_BYTES = 32;
        uint256 internal constant BLOB_FIELD_ELEMENTS = 4096;
        uint256 internal constant BLOB_BYTES = BLOB_FIELD_ELEMENTS * FIELD_ELEMENT_BYTES;

        // ---------------------------------------------------------------
        // Structs
        // ---------------------------------------------------------------

        /// @notice Represents a segment of data that is stored in multiple consecutive blobs created
        /// in this transaction.
        struct BlobReference {
            /// @notice The starting index of the blob.
            uint16 blobStartIndex;
            /// @notice The number of blobs.
            uint16 numBlobs;
            /// @notice The field-element offset within the blob data.
            uint24 offset;
        }

        /// @notice Represents a frame of data that is stored in multiple blobs. Note the size is
        /// encoded as a bytes32 at the offset location.
        struct BlobSlice {
            /// @notice The blobs containing the proposal's content.
            bytes32[] blobHashes;
            /// @notice The byte offset of the proposal's content in the containing blobs.
            uint24 offset;
            /// @notice The timestamp when the frame was created.
            uint48 timestamp;
        }

        // ---------------------------------------------------------------
        // Functions
        // ---------------------------------------------------------------

        /// @dev Validates a blob locator and converts it to a blob slice.
        /// @param _blobReference The blob locator to validate.
        /// @return The blob slice.
        function validateBlobReference(BlobReference memory _blobReference)
            internal
            view
            returns (BlobSlice memory)
        {
            require(_blobReference.numBlobs > 0, NoBlobs());

            bytes32[] memory blobHashes = new bytes32[](_blobReference.numBlobs);
            for (uint256 i; i < _blobReference.numBlobs; ++i) {
                blobHashes[i] = blobhash(_blobReference.blobStartIndex + i);
                require(blobHashes[i] != 0, BlobNotFound());
            }

            return BlobSlice({
                blobHashes: blobHashes,
                offset: _blobReference.offset,
                timestamp: uint48(block.timestamp)
            });
        }

        // ---------------------------------------------------------------
        // Errors
        // ---------------------------------------------------------------

        error BlobNotFound();
        error NoBlobs();
    }

    /// @title IForcedInclusionStore
    /// @custom:security-contact security@taiko.xyz
    #[sol(rpc)]
    #[derive(Debug)]
    interface IForcedInclusionStore {
        /// @notice Represents a forced inclusion that will be stored onchain.
        struct ForcedInclusion {
            /// @notice The fee in Gwei that was paid to submit the forced inclusion.
            uint64 feeInGwei;
            /// @notice The proposal's blob slice.
            LibBlobs.BlobSlice blobSlice;
        }

        /// @dev Event emitted when a forced inclusion is stored.
        event ForcedInclusionSaved(ForcedInclusion forcedInclusion);

        /// @notice Saves a forced inclusion request
        /// A priority fee must be paid to the contract
        /// @param _blobReference The blob locator that contains the transaction data
        function saveForcedInclusion(LibBlobs.BlobReference memory _blobReference) external payable;

        /// @notice Returns the current dynamic forced inclusion fee based on queue size
        /// The fee scales linearly with queue size using the formula:
        /// fee = baseFee × (1 + numPending / threshold)
        /// Examples with threshold = 100 and baseFee = 0.01 ETH:
        /// - 0 pending:   fee = 0.01 × (1 + 0/100)   = 0.01 ETH (1× base)
        /// - 50 pending:  fee = 0.01 × (1 + 50/100)  = 0.015 ETH (1.5× base)
        /// - 100 pending: fee = 0.01 × (1 + 100/100) = 0.02 ETH (2× base, DOUBLED)
        /// - 150 pending: fee = 0.01 × (1 + 150/100) = 0.025 ETH (2.5× base)
        /// - 200 pending: fee = 0.01 × (1 + 200/100) = 0.03 ETH (3× base, TRIPLED)
        /// @return feeInGwei_ The current fee in Gwei
        function getCurrentForcedInclusionFee() external view returns (uint64 feeInGwei_);

        /// @notice Returns forced inclusions stored starting from a given index.
        /// @dev Returns an empty array if `_start` is outside the valid range [head, tail) or if
        ///      `_maxCount` is zero. Otherwise returns actual stored entries from the queue.
        /// @param _start The queue index to start reading from (must be in range [head, tail)).
        /// @param _maxCount Maximum number of inclusions to return. Passing zero returns an empty array.
        /// @return inclusions_ Forced inclusions from the queue starting at `_start`. The actual length
        ///         will be `min(_maxCount, tail - _start)`, or zero if `_start` is out of range.
        function getForcedInclusions(
            uint48 _start,
            uint48 _maxCount
        )
            external
            view
            returns (ForcedInclusion[] memory inclusions_);

        /// @notice Returns the queue pointers for the forced inclusion store.
        /// @return head_ Index of the oldest forced inclusion in the queue.
        /// @return tail_ Index of the next free slot in the queue.
        /// @return lastProcessedAt_ Timestamp when the last forced inclusion was processed.
        function getForcedInclusionState()
            external
            view
            returns (uint48 head_, uint48 tail_);
    }
}
