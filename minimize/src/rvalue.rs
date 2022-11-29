use crate::*;

pub fn translate_rvalue<'tcx>(place: &rs::Rvalue<'tcx>, fcx: FnCtxt<'_, 'tcx>) -> mini::ValueExpr {
    match place {
        rs::Rvalue::Use(operand) => translate_operand(operand, fcx),
        rs::Rvalue::CheckedBinaryOp(bin_op, box (l, r)) => {
            let l = translate_operand(l, fcx);
            let r = translate_operand(r, fcx);

            let l = specr::hidden::GcCow::new(l);
            let r = specr::hidden::GcCow::new(r);

            use rs::BinOp::*;
            let op = if *bin_op == Offset {
                mini::BinOp::PtrOffset {
                    inbounds: true // FIXME where to find this bool `inbouds` in mir?
                }
            } else { // everything else right-now is a int op!
                let op_int = match bin_op {
                    Add => mini::BinOpInt::Add,
                    Sub => mini::BinOpInt::Sub,
                    Mul => mini::BinOpInt::Mul,
                    Div => mini::BinOpInt::Div,
                    x => {
                        dbg!(x);
                        todo!("unsupported BinOp")
                    },
                };
                // FIXME how to detect int-type?
                // fallback to i32.
                let int_ty = mini::IntType {
                    signed: mini::Signedness::Signed,
                    size: specr::Size::from_bits(32),
                };
                mini::BinOp::Int(op_int, int_ty)
            };

            mini::ValueExpr::BinOp {
                operator: op,
                left: l,
                right: r,
            }
        },
        rs::Rvalue::Ref(_, bkind, place) => {
            let place = translate_place(place, fcx);
            let target = specr::hidden::GcCow::new(place);
            let mutbl = translate_mutbl(bkind.to_mutbl_lossy());
            let pointee = todo!(); // TODO how to get layout? Probably using the `place`.
            let ptr_ty = mini::PtrType::Ref { mutbl, pointee };

            mini::ValueExpr::AddrOf { target, ptr_ty }
        },
        x => {
            dbg!(x);
            todo!()
        }
    }
}

pub fn translate_operand<'tcx>(operand: &rs::Operand<'tcx>, fcx: FnCtxt<'_, 'tcx>) -> mini::ValueExpr {
    match operand {
        rs::Operand::Constant(box c) => {
            match c.literal {
                rs::ConstantKind::Val(val, ty) => {
                    let ty = translate_ty(&ty, fcx.tcx);
                    let constant = match ty {
                        mini::Type::Int(int_ty) => {
                            let val = val.try_to_scalar_int().unwrap();

                            use mini::Signedness::*;
                            let bits = specr::hidden::int_to_usize(int_ty.size.bits());
                            // TODO is there no better way to get the value from a ScalarInt?
                            let int: specr::Int = match (int_ty.signed, bits) {
                                (Signed, 8) => val.try_to_i8().unwrap().into(),
                                (Signed, 16) => val.try_to_i16().unwrap().into(),
                                (Signed, 32) => val.try_to_i32().unwrap().into(),
                                (Signed, 64) => val.try_to_i64().unwrap().into(),
                                (Signed, 128) => val.try_to_i128().unwrap().into(),

                                (Unsigned, 8) => val.try_to_u8().unwrap().into(),
                                (Unsigned, 16) => val.try_to_u16().unwrap().into(),
                                (Unsigned, 32) => val.try_to_u32().unwrap().into(),
                                (Unsigned, 64) => val.try_to_u64().unwrap().into(),
                                (Unsigned, 128) => val.try_to_u128().unwrap().into(),
                                _ => panic!("unsupported integer type encountered!"),
                            };
                            mini::Constant::Int(int)
                        },
                        // unit type `()`
                        mini::Type::Tuple { fields, .. } if fields.is_empty() => {
                            mini::Constant::Tuple(specr::List::new())
                        }
                        x => {
                            dbg!(x);
                            todo!()
                        }
                    };
                    mini::ValueExpr::Constant(constant, ty)
                }
                x => {
                    dbg!(x);
                    todo!()
                }
            }
        },
        rs::Operand::Copy(place) => {
            mini::ValueExpr::Load {
                destructive: false,
                source: specr::hidden::GcCow::new(translate_place(place, fcx)),
            }
        },
        rs::Operand::Move(place) => {
            mini::ValueExpr::Load {
                destructive: true,
                source: specr::hidden::GcCow::new(translate_place(place, fcx)),
            }
        },
    }
}

pub fn translate_place<'tcx>(place: &rs::Place<'tcx>, fcx: FnCtxt<'_, 'tcx>) -> mini::PlaceExpr {
    mini::PlaceExpr::Local(fcx.localname_map[&place.local])
    // TODO apply projections
}

