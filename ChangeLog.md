* 1.0.7
	* update readme
* 1.0.6
	* clippy fix
* 1.0.5
    * async_get_object return buffer and use Cow<'a, str> avoid multi alloc
* 1.0.4
	* replace string with raw buffer for the return of get_object
* 1.0.3
	* replace xml-rs with quick_xml
	* refactor service api, return whole ListBuckets info now
	* impl get_object_acl, a example for other object resource api
