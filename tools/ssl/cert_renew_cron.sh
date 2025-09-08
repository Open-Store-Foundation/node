#!/bin/bash

set -e

if [ -z "$VOLUME_DIR" ] || [ -z "$CONFIG_DIR" ]; then
    echo "Error: VOLUME_DIR and CONFIG_DIR environment variables must be set"
    exit 1
fi

echo "Setting up SSL for: CONFIG_DIR $VOLUME_DIR | CONFIG_DIR $CONFIG_DIR"
echo "0 12 * * * cd $(pwd) && VOLUME_DIR=$VOLUME_DIR CONFIG_DIR=$CONFIG_DIR sudo docker compose --profile certbot run --rm certbot renew && VOLUME_DIR=$VOLUME_DIR CONFIG_DIR=$CONFIG_DIR sudo docker compose restart nginx" | sudo crontab -
