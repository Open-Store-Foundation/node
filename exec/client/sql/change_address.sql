UPDATE obj SET address = :NEW_ADDRESS WHERE address = :OLD_ADDRESS;
UPDATE artifact SET asset_address = :NEW_ADDRESS WHERE asset_address = :OLD_ADDRESS;
UPDATE build_request SET asset_address = :NEW_ADDRESS WHERE asset_address = :OLD_ADDRESS;
UPDATE publishing SET asset_address = :NEW_ADDRESS WHERE asset_address = :OLD_ADDRESS;
UPDATE assetlink_sync SET asset_address = :NEW_ADDRESS WHERE asset_address = :OLD_ADDRESS;