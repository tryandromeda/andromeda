// deno-lint-ignore-file no-unused-vars
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

type DOMExceptionName =
  | "IndexSizeError"
  | "HierarchyRequestError"
  | "WrongDocumentError"
  | "InvalidCharacterError"
  | "NoModificationAllowedError"
  | "NotFoundError"
  | "NotSupportedError"
  | "InUseAttributeError"
  | "InvalidStateError"
  | "SyntaxError"
  | "InvalidModificationError"
  | "NamespaceError"
  | "InvalidAccessError"
  | "TypeMismatchError"
  | "SecurityError"
  | "NetworkError"
  | "AbortError"
  | "URLMismatchError"
  | "QuotaExceededError"
  | "TimeoutError"
  | "InvalidNodeTypeError"
  | "DataCloneError";

const DOMExceptionCode: Record<DOMExceptionName, number> = {
  IndexSizeError: 1,
  HierarchyRequestError: 3,
  WrongDocumentError: 4,
  InvalidCharacterError: 5,
  NoModificationAllowedError: 7,
  NotFoundError: 8,
  NotSupportedError: 9,
  InUseAttributeError: 10,
  InvalidStateError: 11,
  SyntaxError: 12,
  InvalidModificationError: 13,
  NamespaceError: 14,
  InvalidAccessError: 15,
  TypeMismatchError: 17,
  SecurityError: 18,
  NetworkError: 19,
  AbortError: 20,
  URLMismatchError: 21,
  QuotaExceededError: 22,
  TimeoutError: 23,
  InvalidNodeTypeError: 24,
  DataCloneError: 25,
};
class DOMException extends Error {
  override readonly name: DOMExceptionName;
  readonly code: number;
  static readonly INDEX_SIZE_ERR = 1;
  static readonly HIERARCHY_REQUEST_ERR = 3;
  static readonly WRONG_DOCUMENT_ERR = 4;
  static readonly INVALID_CHARACTER_ERR = 5;
  static readonly NO_MODIFICATION_ALLOWED_ERR = 7;
  static readonly NOT_FOUND_ERR = 8;
  static readonly NOT_SUPPORTED_ERR = 9;
  static readonly INUSE_ATTRIBUTE_ERR = 10;
  static readonly INVALID_STATE_ERR = 11;
  static readonly SYNTAX_ERR = 12;
  static readonly INVALID_MODIFICATION_ERR = 13;
  static readonly NAMESPACE_ERR = 14;
  static readonly INVALID_ACCESS_ERR = 15;
  static readonly TYPE_MISMATCH_ERR = 17;
  static readonly SECURITY_ERR = 18;
  static readonly NETWORK_ERR = 19;
  static readonly ABORT_ERR = 20;
  static readonly URL_MISMATCH_ERR = 21;
  static readonly QUOTA_EXCEEDED_ERR = 22;
  static readonly TIMEOUT_ERR = 23;
  static readonly INVALID_NODE_TYPE_ERR = 24;
  static readonly DATA_CLONE_ERR = 25;

  constructor(
    message?: string,
    name: DOMExceptionName = "InvalidStateError",
  ) {
    super(message);
    this.name = name;
    this.code = DOMExceptionCode[name] || 0;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

function createDOMException(
  message?: string,
  name: DOMExceptionName = "InvalidStateError",
): DOMException {
  return new DOMException(message, name);
}
