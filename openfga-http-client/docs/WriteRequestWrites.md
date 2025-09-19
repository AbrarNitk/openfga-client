# WriteRequestWrites

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tuple_keys** | [**Vec<models::TupleKey>**](TupleKey.md) |  | 
**on_duplicate** | Option<**String**> | On 'error' ( or unspecified ), the API returns an error if an identical tuple already exists. On 'ignore', identical writes are treated as no-ops (matching on user, relation, object, and RelationshipCondition). | [optional][default to Error]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


