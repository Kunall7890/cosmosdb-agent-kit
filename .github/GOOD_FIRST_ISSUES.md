# Good First Issues — Ready to File

Copy each issue below into GitHub Issues with labels: `good first issue`, `help-wanted`.
These are self-contained, single-rule tasks with clear guidance — perfect for first-time contributors.

---

## Issue 1: Add rule `query-use-top-clause.md`

**Title:** [Rule] Use TOP clause to limit cross-partition query results

**Body:**
We're missing a rule that advises developers to always use `TOP n` (or OFFSET/LIMIT) when querying across partitions to avoid unbounded result sets that consume excessive RUs.

**Category:** `query-` Query Optimization
**Impact:** High
**Suggested file:** `skills/cosmosdb-best-practices/rules/query-use-top-clause.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/nosql/query/offset-limit

---

## Issue 2: Add rule `sdk-singleton-client.md`

**Title:** [Rule] Use a singleton CosmosClient instance per application lifetime

**Body:**
A common mistake is creating a new `CosmosClient` on every request. This wastes TCP connections and increases latency. We need a rule that enforces the singleton pattern.

**Category:** `sdk-` SDK Best Practices
**Impact:** Critical
**Suggested file:** `skills/cosmosdb-best-practices/rules/sdk-singleton-client.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/nosql/best-practice-dotnet

---

## Issue 3: Add rule `sdk-retry-429.md`

**Title:** [Rule] Let the SDK handle 429 retries — don't retry manually

**Body:**
The Cosmos DB SDKs have built-in retry logic for throttled requests (HTTP 429). Developers sometimes add manual retry loops on top, causing duplicate work. We need a rule to clarify this.

**Category:** `sdk-` SDK Best Practices
**Impact:** Medium
**Suggested file:** `skills/cosmosdb-best-practices/rules/sdk-retry-429.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/nosql/conceptual-resilient-sdk-applications

---

## Issue 4: Add rule `query-avoid-like-startswith.md`

**Title:** [Rule] Prefer StartsWith() over LIKE for prefix matching

**Body:**
`LIKE 'prefix%'` can't always use the index efficiently. `STARTSWITH(field, 'prefix')` is optimized. We should advise this.

**Category:** `query-` Query Optimization
**Impact:** Medium
**Suggested file:** `skills/cosmosdb-best-practices/rules/query-avoid-like-startswith.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/nosql/query/keywords#like

---

## Issue 5: Add rule `model-ttl-expiration.md`

**Title:** [Rule] Use TTL for automatic data expiration instead of manual deletion

**Body:**
Many developers write scheduled jobs to delete old data. Cosmos DB's built-in TTL is more efficient and costs nothing in RUs. We need a rule for this.

**Category:** `model-` Data Modeling
**Impact:** Medium
**Suggested file:** `skills/cosmosdb-best-practices/rules/model-ttl-expiration.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/nosql/time-to-live

---

## Issue 6: Add rule `sdk-prefer-stream-api.md`

**Title:** [Rule] Use Stream APIs for read-heavy workloads to reduce deserialization cost

**Body:**
The .NET SDK's `ReadItemStreamAsync` and `GetItemQueryStreamIterator` skip deserialization, reducing CPU and memory. We should recommend this for high-throughput scenarios.

**Category:** `sdk-` SDK Best Practices
**Impact:** High
**Suggested file:** `skills/cosmosdb-best-practices/rules/sdk-prefer-stream-api.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/nosql/best-practice-dotnet#use-stream-api

---

## Issue 7: Add rule `partition-synthetic-key.md`

**Title:** [Rule] Use synthetic partition keys to combine multiple fields

**Body:**
When no single property provides good cardinality, concatenating fields into a synthetic key (e.g., `userId-orderId`) is the recommended pattern. We need a rule for this.

**Category:** `partition-` Partition Key Design
**Impact:** High
**Suggested file:** `skills/cosmosdb-best-practices/rules/partition-synthetic-key.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/nosql/synthetic-partition-keys

---

## Issue 8: Add rule `throughput-autoscale-vs-manual.md`

**Title:** [Rule] Use autoscale throughput for unpredictable workloads

**Body:**
Manual throughput requires knowing peak RU/s upfront. Autoscale adapts to traffic spikes without throttling. We should advise when to use each.

**Category:** `throughput-` Throughput & Scaling
**Impact:** Medium
**Suggested file:** `skills/cosmosdb-best-practices/rules/throughput-autoscale-vs-manual.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/provision-throughput-autoscale

---

## Issue 9: Add rule `security-rbac-over-keys.md`

**Title:** [Rule] Prefer Microsoft Entra RBAC over primary/secondary keys

**Body:**
Master keys grant full access and can't be scoped. RBAC with managed identities is more secure and auditable. This should be a security rule.

**Category:** `security-` Security
**Impact:** High
**Suggested file:** `skills/cosmosdb-best-practices/rules/security-rbac-over-keys.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/role-based-access-control

---

## Issue 10: Add rule `query-project-only-needed-fields.md`

**Title:** [Rule] Project only required fields in queries to reduce RU cost

**Body:**
`SELECT *` returns the full document which increases bandwidth and RU cost. Projecting only needed properties (`SELECT c.id, c.name FROM c`) is cheaper.

**Category:** `query-` Query Optimization
**Impact:** Medium
**Suggested file:** `skills/cosmosdb-best-practices/rules/query-project-only-needed-fields.md`

References:
- https://learn.microsoft.com/azure/cosmos-db/nosql/query-metrics#retrieved-document-size
