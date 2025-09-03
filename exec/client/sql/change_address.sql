UPDATE obj SET address = :NEW_ADDRESS WHERE address = :OLD_ADDRESS;
UPDATE artifact SET object_address = :NEW_ADDRESS WHERE object_address = :OLD_ADDRESS;
UPDATE build_request SET object_address = :NEW_ADDRESS WHERE object_address = :OLD_ADDRESS;
UPDATE publishing SET object_address = :NEW_ADDRESS WHERE object_address = :OLD_ADDRESS;
UPDATE assetlink_sync SET object_address = :NEW_ADDRESS WHERE object_address = :OLD_ADDRESS;