#[derive(Fail, Debug)]
pub enum ObjectError {
	#[fail(display = "PUT ERROR: {}", _0)]
	PutError(String),
	#[fail(display = "GET ERROR: {}", _0)]
	GetError(String),
	#[fail(display = "COPY ERROR: {}", _0)]
	CopyError(String),
	#[fail(display = "DELETE ERROR: {}", _0)]
	DeleteError(String)
}