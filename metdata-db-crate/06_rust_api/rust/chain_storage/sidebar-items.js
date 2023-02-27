window.SIDEBAR_ITEMS = {"enum":[["ConsistencyFailure",""],["Error",""]],"mod":[["test_utils","Utilities for testing the storage."]],"struct":[["BlockInfo","A structure that holds the information about a blocks, that is needed to maintain consistency of the storage. This include the ID of the blocks, the ID of its parent and the length of the block chain for the given block."],["BlockStore",""],["StorageIterator","Iterator over blocks. Starts from n-th ancestor of the given block."],["Value","Wrapper for data held by the database. This wrapper holds structs returned by both volatile and permanent storage to ensure we don’t have needless copying on return. Data should be accessed through the `AsRef` trait."]]};