"""Sample Cosmos DB application code for eval fixtures."""
from azure.cosmos import CosmosClient, PartitionKey


# Anti-pattern: creating client per request
def get_order(connection_string, order_id, customer_id):
    client = CosmosClient.from_connection_string(connection_string)
    database = client.get_database_client("ecommerce")
    container = database.get_container_client("orders")

    # Anti-pattern: using query instead of point read
    query = f"SELECT * FROM c WHERE c.id = '{order_id}'"
    items = list(container.query_items(query=query, enable_cross_partition_query=True))
    return items[0] if items else None


def get_all_orders():
    """Anti-pattern: cross-partition scan without filters."""
    client = CosmosClient.from_connection_string("...")
    container = client.get_database_client("ecommerce").get_container_client("orders")
    return list(container.query_items(
        query="SELECT * FROM c",
        enable_cross_partition_query=True
    ))

