mod utils;
mod web4;

use crate::utils::unordered_map_pagination;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub applications: UnorderedMap<AccountId, Option<ApplicationData>>,
    pub posts: UnorderedMap<u64, Post>,
    pub tips: UnorderedMap<AccountId, u128>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Post {
    pub title: String,
    pub text: String,
    pub author: AccountId,
    pub created_at: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ApplicationData {
    pub description: String,
    pub github_url: String,
    pub contact_data: String,
    pub contract_id: String,
    pub youtube_url: Option<String>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Applications,
    Posts,
    Tips,
}

impl Default for Contract {
    fn default() -> Self {
      Self {
        applications: UnorderedMap::new(StorageKey::Applications),
        posts: UnorderedMap::new(StorageKey::Posts),
        tips: UnorderedMap::new(StorageKey::Tips),
      }
    }
  }

#[near_bindgen]
impl Contract {
    pub fn add_post(&mut self, title: String, text: String) -> bool {
        assert!(title.len() > 0, "Title is reqired.");
        assert!(text.len() > 0, "Text is required.");
        let new_post = Post {
            title: title,
            text: text,
            author: env::predecessor_account_id(),
            created_at: env::block_timestamp(),
        };
        let id = self.posts.len();
        self.posts.insert(&id, &new_post);
        true
    }

    pub fn update_post(&mut self, post_id: u64, title: String, text: String) -> bool {
        let editor_id = env::predecessor_account_id();
        let post = self.get_post(post_id);
        assert_eq!(editor_id, post.author, "Only author can update the post.");
        let updated_post = Post {
            title: title,
            text: text,
            ..post
        };
        self.posts.insert(&post_id, &updated_post);
        true
    }

    #[payable]
    pub fn tip_author(&mut self, author_id: AccountId) -> bool {
        let deposit = env::attached_deposit();
        // let donator_account_id: AccountId = env::predecessor_account_id();
        assert!(deposit > 0, "The amount of tips should be greater than 0");
        let total = self.tips.get(&author_id).unwrap_or(0) + deposit;
        self.tips.insert(&author_id, &total);
        true
    }

    pub fn withdraw_tips(&mut self) -> Promise {
        let account_id: AccountId = env::predecessor_account_id();
        let withdrowal_amount = self.tips.get(&account_id).unwrap_or(0);
        assert!(
            withdrowal_amount > 0,
            "Withdrowal amount should be greater than 0"
        );
        self.tips.remove(&account_id);
        Promise::new(account_id).transfer(withdrowal_amount)
    }

    /// Getters

    pub fn get_posts(&self) -> Vec<(u64, Post)> {
        self.posts.iter().collect()
    }

    pub fn get_post(&self, post_id: u64) -> Post {
        assert!(
            self.posts.get(&post_id).is_some(),
            "Post with id {} not found.",
            &post_id
        );
        self.posts.get(&post_id).unwrap()
    }

    pub fn get_tips(&self, author_id: AccountId) -> u128 {
        self.tips.get(&author_id).unwrap_or(0)
    }
}

// Tests

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(account: String, is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(account.parse().unwrap())
            .attached_deposit(1_000_000_000_000_000_000_000)
            .is_view(is_view)
            .build()
    }

  #[test]
  fn add_then_get_posts() {
    let context = get_context("bob_near".to_string(), false);
    testing_env!(context);
    let mut contract = Contract::default();
    let add = contract.add_post(
      "title".to_string(),
      "text".to_string(),
    );
    let posts = contract.get_posts();
    assert!(add);
    assert_eq!(1, posts.len());
    let post = contract.get_post(0);
    assert_eq!("title".to_string(), post.title);
    assert_eq!("text".to_string(), post.text);
    assert_eq!("bob_near".parse::<AccountId>().unwrap(), post.author);

  }

  #[test]
  #[should_panic(expected="Title is reqired.")]
  fn title_is_empty() {
    let context = get_context("bob_near".to_string(), false);
    testing_env!(context);
    let mut contract = Contract::default();
    contract.add_post(
      "".to_string(),
      "text".to_string(),
    );
  }

  #[test]
  #[should_panic(expected="Text is required.")]
  fn text_is_empty() {
    let context = get_context("bob_near".to_string(), false);
    testing_env!(context);
    let mut contract = Contract::default();
    contract.add_post(
      "title".to_string(),
      "".to_string(),
    );
  }

  #[test]
  #[should_panic(expected="Only author can update the post.")]
  fn cannot_update_if_not_an_author() {
    let context = get_context("bob_near".to_string(), false);
    testing_env!(context);
    let mut contract = Contract::default();
    contract.add_post(
      "title".to_string(),
      "text".to_string(),
    );
    let new_context = get_context("john_near".to_string(), false);
    testing_env!(new_context);
    contract.update_post(0, "new title".to_string(), "new text".to_string());
  }

  #[test]
  fn update_and_get_post() {
    let context = get_context("bob_near".to_string(), false);
    testing_env!(context);
    let mut contract = Contract::default();
    contract.add_post(
      "title".to_string(),
      "text".to_string(),
    );
    let update = contract.update_post(0, "new title".to_string(), "new text".to_string());
    assert!(update);
    let posts = contract.get_posts();
    assert_eq!(1, posts.len());
    let post = contract.get_post(0);
    assert_eq!("new title".to_string(), post.title);
    assert_eq!("new text".to_string(), post.text);
  }

  #[test]
  fn tip_author() {
    let context = get_context("alice_near".to_string(), false);
    testing_env!(context);
    let mut contract = Contract::default();
    assert_eq!(contract.get_tips("bob_near".parse().unwrap()), 0);
    let tip = contract.tip_author("bob_near".parse().unwrap());
    assert!(tip);
    assert_eq!(contract.get_tips("bob_near".parse().unwrap()), 1_000_000_000_000_000_000_000);
  }

  #[test]
  fn withraw_tips() {
    let context = get_context("bob_near".to_string(), false);
    testing_env!(context);
    let mut contract = Contract::default();
    let tip = contract.tip_author("sam_near".parse().unwrap());
    assert!(tip);
    assert_eq!(contract.get_tips("sam_near".parse().unwrap()), 1_000_000_000_000_000_000_000);
    let new_context = get_context("sam_near".to_string(), false);
    testing_env!(new_context);
    contract.withdraw_tips();
    assert_eq!(contract.get_tips("sam_near".parse().unwrap()), 0);
  }

  #[test]
  #[should_panic(expected="Withdrowal amount should be greater than 0")]
  fn cannot_withraw_0() {
    let context = get_context("bob_near".to_string(), false);
    testing_env!(context);
    let mut contract = Contract::default();
    assert_eq!(contract.get_tips("bob_near".parse().unwrap()), 0);
    contract.withdraw_tips();
  }
}

