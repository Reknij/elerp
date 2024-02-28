import { web } from "..";
import { ListSlice } from "../models";
import {
  UserInfo,
  GetUsersQuery,
  GetTokenQuery,
  AuthenticatedUser,
  UserConfigure,
} from "./models";

export async function add_user(v: UserInfo): Promise<UserInfo> {
  const resp = await web.post("/us/users", v);
  return resp.data;
}

export async function remove_user(id: number): Promise<boolean> {
  const resp = await web.delete(`/us/users/${id}`);
  return resp.status == 200;
}

export async function get_user(id: number): Promise<UserInfo> {
  const resp = await web.get(`/us/users/${id}`);
  return resp.data;
}

export async function get_users(
  q: GetUsersQuery
): Promise<ListSlice<UserInfo>> {
  const resp = await web.get(`/us/users`, {
    params: q,
  });
  return resp.data;
}

export async function update_user(id: number, v: UserInfo): Promise<UserInfo> {
  const resp = await web.put(`/us/users/${id}`, v);
  return resp.data;
}

export async function get_user_token(
  query: GetTokenQuery
): Promise<AuthenticatedUser> {
  const resp = await web.get(`/us/users_token`, {
    params: query,
  });
  return resp.data;
}

export async function get_me(): Promise<AuthenticatedUser> {
  const resp = await web.get(`/us/me`);
  return resp.data;
}

export async function remove_user_token(id: number): Promise<boolean> {
  const resp = await web.delete(`/us/users_token/${id}`);
  return resp.status == 200;
}

export async function get_user_configure(id: number): Promise<UserConfigure> {
  const resp = await web.get(`/us/users_configure/${id}`);
  return resp.data;
}

export async function update_user_configure(id: number, v: UserConfigure): Promise<UserConfigure> {
  const resp = await web.put(`/us/users_configure/${id}`, v);
  return resp.data;
}
