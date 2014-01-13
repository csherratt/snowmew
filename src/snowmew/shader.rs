
#[deriving(Clone, Default)]
pub struct Shader
{
	vertex: ~str,
	geometry: Option<~str>,
	frag: ~str,
}

impl Shader {
	pub fn new(vertex: ~str, frag: ~str) -> Shader
	{
		Shader {
			vertex: vertex,
			geometry: None,
			frag: frag
		}
	}
}