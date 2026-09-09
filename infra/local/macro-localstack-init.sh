#!/usr/bin/env bash

# Macro's local profile uses LocalStack instead of AWS.  The LocalStack
# container is intentionally disposable, so this hook must recreate every
# resource the generated Macro environment references whenever LocalStack
# becomes ready.  It is safe to run repeatedly.

set -u

retry() {
  local attempts="$1"
  shift

  local attempt=1
  while ! "$@"; do
    if (( attempt >= attempts )); then
      echo "LocalStack provisioning command failed after ${attempts} attempts: $*" >&2
      return 1
    fi
    sleep 2
    attempt=$((attempt + 1))
  done
}

wait_for_services() {
  echo "Waiting for LocalStack APIs"
  retry 60 awslocal sqs list-queues >/dev/null
  retry 60 awslocal s3api list-buckets >/dev/null
  retry 60 awslocal dynamodb list-tables >/dev/null
}

ensure_bucket() {
  local bucket="$1"

  if ! awslocal s3api head-bucket --bucket "$bucket" >/dev/null 2>&1; then
    if ! retry 30 awslocal s3 mb "s3://$bucket" >/dev/null 2>&1; then
      retry 30 awslocal s3api head-bucket --bucket "$bucket" >/dev/null
    fi
  fi

  retry 30 awslocal s3api put-bucket-cors \
    --bucket "$bucket" \
    --cors-configuration "$cors" >/dev/null
}

ensure_table() {
  local table="$1"
  shift

  if awslocal dynamodb describe-table --table-name "$table" >/dev/null 2>&1; then
    return 0
  fi

  if ! retry 30 awslocal dynamodb create-table "$@" >/dev/null 2>&1; then
    retry 30 awslocal dynamodb describe-table --table-name "$table" >/dev/null
  fi
}

wait_for_services

queues=(
  notification-queue
  notification-ingress-queue
  push-delivery-queue
  bulk-upload-queue
  webhook-event-queue.fifo
  email-service-backfill-queue
  email-service-crm-cleanup-queue
  delete-chat-handler-queue
  contacts-queue
  convert-service-queue
  delete-document-handler-queue
  document-upload-finalizer-queue
  document-text-extractor-lambda-queue
  email-service-scheduled-queue
  email-service-gmail-inbox-sync-queue
  email-service-gmail-inbox-retry-queue
  email-service-gmail-ops-queue
  email-service-gmail-ops-retry-queue
  email-service-refresh-queue
  search-event-queue
  ai-projection-queue
  email-sfs-delete-queue
  email-service-sfs-mapper-queue
  static-file-s3-event-notification-queue
  reminder-dispatch-queue
  calendar-reminder-dispatch-queue
  organization-retention-handler-queue
  notification-queue-prod
  notification-ingress-queue-prod
  push-delivery-queue-prod
  bulk-upload-queue-prod
  webhook-event-queue-prod.fifo
  email-service-backfill-queue-prod
  email-service-crm-cleanup-queue-prod
  delete-chat-handler-queue-prod
  contacts-queue-prod
  convert-service-queue-prod
  delete-document-handler-queue-prod
  document-upload-finalizer-queue-prod
  document-text-extractor-lambda-queue-prod
  email-service-scheduled-queue-prod
  email-service-gmail-webhook-queue-prod
  email-service-gmail-webhook-retry-queue-prod
  email-service-gmail-ops-queue-prod
  email-service-gmail-ops-retry-queue-prod
  email-service-refresh-queue-prod
  search-event-queue-prod
  ai-projection-queue-prod
  email-sfs-delete-queue-prod
  email-service-sfs-mapper-queue-prod
  static-file-s3-event-notification-queue-prod
  reminder-dispatch-queue-prod
  calendar-reminder-dispatch-queue-prod
  organization-retention-handler-queue-prod
)

for queue in "${queues[@]}"; do
  if [[ "$queue" == *.fifo ]]; then
    retry 30 awslocal sqs create-queue \
      --queue-name "$queue" \
      --attributes FifoQueue=true >/dev/null
  else
    retry 30 awslocal sqs create-queue --queue-name "$queue" >/dev/null
  fi
done

buckets=(
  macro-email-attachments
  doc-storage
  docx-upload
  static-file-storage
  bulk-upload-staging
  macro-call-recording-local
)

cors='{"CORSRules":[{"AllowedOrigins":["*"],"AllowedMethods":["GET","PUT","POST","DELETE","HEAD"],"AllowedHeaders":["*"],"ExposeHeaders":["ETag"],"MaxAgeSeconds":3600}]}'
for bucket in "${buckets[@]}"; do
  ensure_bucket "$bucket"
done

ensure_table bulk-upload \
  --table-name bulk-upload \
  --attribute-definitions AttributeName=PK,AttributeType=S AttributeName=SK,AttributeType=S \
  --key-schema AttributeName=PK,KeyType=HASH AttributeName=SK,KeyType=RANGE \
  --billing-mode PAY_PER_REQUEST \
  --global-secondary-indexes '[{"IndexName":"DocumentPkIndex","KeySchema":[{"AttributeName":"SK","KeyType":"HASH"}],"Projection":{"ProjectionType":"ALL"}}]' \
  >/dev/null

ensure_table connection-gateway-table \
  --table-name connection-gateway-table \
  --attribute-definitions AttributeName=PK,AttributeType=S AttributeName=SK,AttributeType=S \
  --key-schema AttributeName=PK,KeyType=HASH AttributeName=SK,KeyType=RANGE \
  --billing-mode PAY_PER_REQUEST \
  --global-secondary-indexes '[{"IndexName":"ConnectionPkIndex","KeySchema":[{"AttributeName":"SK","KeyType":"HASH"},{"AttributeName":"PK","KeyType":"RANGE"}],"Projection":{"ProjectionType":"ALL"}}]' \
  >/dev/null

ensure_table static-file-metadata \
  --table-name static-file-metadata \
  --attribute-definitions AttributeName=file_id,AttributeType=S \
  --key-schema AttributeName=file_id,KeyType=HASH \
  --billing-mode PAY_PER_REQUEST \
  >/dev/null

notification='{"QueueConfigurations":[{"Id":"document-upload-finalizer","QueueArn":"arn:aws:sqs:us-east-1:000000000000:document-upload-finalizer-queue","Events":["s3:ObjectCreated:*"]}]}'
retry 30 awslocal s3api put-bucket-notification-configuration \
  --bucket doc-storage \
  --notification-configuration "$notification" >/dev/null

echo "Macro LocalStack resources provisioned"
