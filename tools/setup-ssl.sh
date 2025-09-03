#!/bin/bash

set -e

if [ -z "$DOMAIN_NAME" ] || [ -z "$CERTBOT_EMAIL" ]; then
    echo "Error: DOMAIN_NAME and CERTBOT_EMAIL environment variables must be set"
    echo "Example: DOMAIN_NAME=api.openstore.com CERTBOT_EMAIL=admin@openstore.com ./setup-ssl.sh"
    exit 1
fi

echo "Setting up SSL for domain: $DOMAIN_NAME"
echo "Using email: $CERTBOT_EMAIL"

echo "Step 2: Using initial template (HTTP only)..."
./nginx-template.sh initial

echo "Step 3: Starting nginx for initial setup..."
DOMAIN_NAME=$DOMAIN_NAME docker compose up -d nginx

echo "Step 4: Waiting for nginx to be ready..."
sleep 10

echo "Step 5: Obtaining SSL certificate..."
CERTBOT_EMAIL=$CERTBOT_EMAIL DOMAIN_NAME=$DOMAIN_NAME docker compose --profile certbot run --rm certbot

echo "Step 6: Switching to SSL template..."
./nginx-template.sh ssl

echo "Step 7: Restarting nginx with SSL config..."
DOMAIN_NAME=$DOMAIN_NAME docker compose restart nginx

echo "Step 8: Setting up certificate renewal cron job..."
echo "0 12 * * * cd $(pwd) && DOMAIN_NAME=$DOMAIN_NAME docker compose --profile certbot run --rm certbot renew && DOMAIN_NAME=$DOMAIN_NAME docker compose restart nginx" | crontab -

echo "SSL setup complete! Your API is now available at https://$DOMAIN_NAME"
echo "Certificate will auto-renew daily at 12:00 PM"
