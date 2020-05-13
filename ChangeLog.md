* 0.2.0
	* embrace async/await
* 0.1.10
	* more elegant api for str or String parameters
* 0.1.8
	* not use failure anymore
* 0.1.7
	* update readme
* 0.1.6
	* clippy fix
* 0.1.5
    * async_get_object return buffer and use Cow<'a, str> avoid multi alloc
* 0.1.4
	* replace string with raw buffer for the return of get_object
* 0.1.3
	* replace xml-rs with quick_xml
	* refactor service api, return whole ListBuckets info now
	* impl get_object_acl, a example for other object resource api
