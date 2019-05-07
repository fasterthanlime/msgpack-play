package main

import (
	"bytes"
	"fmt"
	"io/ioutil"

	"github.com/k0kubun/pp"
	"github.com/pkg/errors"
	"github.com/vmihailenco/msgpack"
	"github.com/vmihailenco/msgpack/codes"
)

type Profile_LoginWithPassword_Params struct {
	Username string `json:"username"`
	Password string `json:"password"`
	Bytes    []byte `json:"bytes"`
}

func DecodeMessage(d *msgpack.Decoder) (interface{}, error) {
	c, err := d.PeekCode()
	if err != nil {
		return nil, errors.WithStack(err)
	}
	if !codes.IsFixedArray(c) {
		return nil, errors.New("invalid msgpack-RPC message")
	}

	_, err = d.DecodeArrayLen()
	if err != nil {
		return nil, errors.WithStack(err)
	}

	kind, err := d.DecodeUint32()
	if err != nil {
		return nil, errors.WithStack(err)
	}

	switch kind {
	case 0:
	default:
		return nil, errors.Errorf("unknown messagepack-RPC message type %d", kind)
	}

	id, err := d.DecodeUint32()
	if err != nil {
		return nil, errors.WithStack(err)
	}

	method, err := d.DecodeString()
	if err != nil {
		return nil, errors.WithStack(err)
	}

	var params interface{}
	switch method {
	case "Profile.LoginWithPassword":
		var p Profile_LoginWithPassword_Params
		err = d.Decode(&p)
		if err != nil {
			return nil, errors.WithStack(err)
		}
		params = p
	default:
		return nil, errors.Errorf("unknown method: %s", method)
	}

	return Request{
		ID:     id,
		Method: method,
		Params: params,
	}, nil
}

type Request struct {
	ID     uint32
	Method string
	Params interface{}
}

type Response struct {
	ID     uint32
	Error  string
	Result interface{}
}

func main() {
	path := "./buf.bin"
	fmt.Printf("Reading from %s\n", path)

	bs, err := ioutil.ReadFile(path)
	must(err)

	dec := msgpack.NewDecoder(bytes.NewReader(bs))
	dec.UseJSONTag(true)

	msg, err := DecodeMessage(dec)
	must(err)
	pp.Print(msg)

	fmt.Println()
}

func must(err error) {
	if err != nil {
		panic(fmt.Sprintf("%+v", err))
	}
}
