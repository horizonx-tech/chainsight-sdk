#[derive(Default, Clone, Debug, PartialEq)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub timestamp: u64,
    pub content: String,
    pub version: u8,
}

mod manage_single_state {
    use super::Message;
    use chainsight_cdk_macros::manage_single_state;

    manage_single_state!("last_timestamp", u64, false, 100);
    manage_single_state!("latest_result", String, false);
    manage_single_state!("latest_message", Message, false);

    #[test]
    fn test_u64() {
        assert_eq!(get_last_timestamp(), 100);
        set_last_timestamp(200);
        assert_eq!(get_last_timestamp(), 200);
    }

    #[test]
    fn test_string() {
        assert_eq!(get_latest_result(), String::from(""));
        set_latest_result(String::from("UPDATED"));
        assert_eq!(get_latest_result(), String::from("UPDATED"));
    }

    #[test]
    fn test_custom_struct() {
        assert_eq!(get_latest_message(), Message::default());
        let msg = Message {
            from: String::from("from"),
            to: String::from("to"),
            timestamp: 10,
            content: String::from("content"),
            version: 1,
        };
        set_latest_message(msg.clone());
        assert_eq!(get_latest_message(), msg.clone());
    }
}

mod manage_vec_state {
    use super::Message;
    use chainsight_cdk_macros::manage_vec_state;

    manage_vec_state!("result", String, false);
    manage_vec_state!("message", Message, false);

    #[test]
    fn test_vec() {
        assert_eq!(results_len(), 0);
        let datum1 = String::from("RESULT1");
        let datum2 = String::from("RESULT2");
        let datum3 = String::from("RESULT3");
        add_result(datum1.clone());
        add_result(datum2.clone());
        add_result(datum3.clone());
        assert_eq!(results_len(), 3);
        assert_eq!(
            get_results(),
            vec![datum1.clone(), datum2.clone(), datum3.clone()]
        );
        assert_eq!(get_last_result(), datum3.clone());
        assert_eq!(get_top_results(2), vec![datum3.clone(), datum2.clone()]);
        assert_eq!(get_result(0), datum1.clone());
        assert_eq!(get_result(1), datum2.clone());
    }

    #[test]
    fn test_custom_struct() {
        assert_eq!(messages_len(), 0);
        let datum1 = Message {
            from: String::from("from1"),
            to: String::from("to1"),
            timestamp: 10,
            content: String::from("content1"),
            version: 1,
        };
        let datum2 = Message {
            from: String::from("from2"),
            to: String::from("to2"),
            timestamp: 20,
            content: String::from("content2"),
            version: 2,
        };
        let datum3 = Message {
            from: String::from("from3"),
            to: String::from("to3"),
            timestamp: 30,
            content: String::from("content3"),
            version: 3,
        };
        add_message(datum1.clone());
        add_message(datum2.clone());
        add_message(datum3.clone());
        assert_eq!(messages_len(), 3);
        assert_eq!(
            get_messages(),
            vec![datum1.clone(), datum2.clone(), datum3.clone()]
        );
        assert_eq!(get_last_message(), datum3.clone());
        assert_eq!(get_top_messages(2), vec![datum3.clone(), datum2.clone()]);
        assert_eq!(get_message(0), datum1.clone());
        assert_eq!(get_message(1), datum2.clone());
    }
}

mod manage_map_state {
    use super::Message;
    use chainsight_cdk_macros::manage_map_state;

    manage_map_state!("balance", String, u64, false);
    manage_map_state!("username", u64, String, false);
    manage_map_state!("message", u64, Message, false);

    #[test]
    fn test_balances() {
        assert_eq!(balances_len(), 0);
        let datum1 = String::from("BALANCE1");
        let datum2 = String::from("BALANCE2");
        insert_balance(datum1.clone(), 100);
        insert_balance(datum2.clone(), 200);
        assert_eq!(balances_len(), 2);
        assert_eq!(get_balance(datum1.clone()), 100);
        assert_eq!(get_balance(datum2.clone()), 200);
    }

    #[test]
    fn test_usernames() {
        assert_eq!(usernames_len(), 0);
        let datum1 = String::from("USERNAME1");
        let datum2 = String::from("USERNAME2");
        insert_username(1, datum1.clone());
        insert_username(2, datum2.clone());
        assert_eq!(usernames_len(), 2);
        assert_eq!(get_username(1), datum1.clone());
        assert_eq!(get_username(2), datum2.clone());
    }

    #[test]
    fn test_messages() {
        assert_eq!(messages_len(), 0);
        let datum1 = Message {
            from: String::from("from1"),
            to: String::from("to1"),
            timestamp: 10,
            content: String::from("content1"),
            version: 1,
        };
        let datum2 = Message {
            from: String::from("from2"),
            to: String::from("to2"),
            timestamp: 20,
            content: String::from("content2"),
            version: 2,
        };
        insert_message(1, datum1.clone());
        insert_message(2, datum2.clone());
        assert_eq!(messages_len(), 2);
        assert_eq!(get_message(1), datum1.clone());
        assert_eq!(get_message(2), datum2.clone());
    }
}
