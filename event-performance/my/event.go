package my

type Event struct {
	Times   []int64 `json:"times"`
	Data    []byte  `json:"data"`
	DataRef string  `json:"dataref"`
}
