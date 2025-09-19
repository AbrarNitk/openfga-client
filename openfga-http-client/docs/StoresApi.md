# \StoresApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_store**](StoresApi.md#create_store) | **POST** /stores | Create a store
[**delete_store**](StoresApi.md#delete_store) | **DELETE** /stores/{store_id} | Delete a store
[**get_store**](StoresApi.md#get_store) | **GET** /stores/{store_id} | Get a store
[**list_stores**](StoresApi.md#list_stores) | **GET** /stores | List all stores



## create_store

> models::CreateStoreResponse create_store(body)
Create a store

Create a unique OpenFGA store which will be used to store authorization models and relationship tuples.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body** | [**CreateStoreRequest**](CreateStoreRequest.md) |  | [required] |

### Return type

[**models::CreateStoreResponse**](CreateStoreResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_store

> delete_store(store_id)
Delete a store

Delete an OpenFGA store. This does not delete the data associated with the store, like tuples or authorization models.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_store

> models::GetStoreResponse get_store(store_id)
Get a store

Returns an OpenFGA store by its identifier

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** |  | [required] |

### Return type

[**models::GetStoreResponse**](GetStoreResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_stores

> models::ListStoresResponse list_stores(page_size, continuation_token, name)
List all stores

Returns a paginated list of OpenFGA stores and a continuation token to get additional stores. The continuation token will be empty if there are no more stores. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | Option<**i32**> |  |  |
**continuation_token** | Option<**String**> |  |  |
**name** | Option<**String**> | The name parameter instructs the API to only include results that match that name.Multiple results may be returned. Only exact matches will be returned; substring matches and regexes will not be evaluated |  |

### Return type

[**models::ListStoresResponse**](ListStoresResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

