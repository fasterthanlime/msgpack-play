package main

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"log"

	"github.com/pkg/errors"
	"github.com/vmihailenco/msgpack"
	"github.com/vmihailenco/msgpack/codes"
)

type RPCMessage struct {
	Request  *Request
	Response *Response
}

var _ msgpack.CustomDecoder = (*RPCMessage)(nil)

func (m *RPCMessage) DecodeMsgpack(d *msgpack.Decoder) error {
	c, err := d.PeekCode()
	if err != nil {
		return errors.WithStack(err)
	}
	log.Printf("peeked code: %#v", c)
	if codes.IsFixedMap(c) {
		log.Printf("is fixed map!")
	}

	mymap, err := d.DecodeMap()
	if err != nil {
		return errors.WithStack(err)
	}

	log.Printf("mymap = %#v", mymap)

	return nil
}

type Request struct {
	ID     uint32 `json:"id"`
	Type   string `json:"type"`
	Params Params `json:"params"`
}

type Response struct {
	ID    uint32 `json:"id"`
	Error string `json:"error"`
}

type Params struct {
	Method string `json:"method"`
}

func main() {
	bs, err := ioutil.ReadFile("./buf.bin")
	must(err)

	var msg RPCMessage
	dec := msgpack.NewDecoder(bytes.NewReader(bs))
	dec.UseJSONTag(true)
	must(dec.Decode(&msg))

	log.Printf("msg = %#v", msg)
}

func must(err error) {
	if err != nil {
		panic(fmt.Sprintf("%+v", err))
	}
}
