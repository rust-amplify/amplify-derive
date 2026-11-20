#[automatically_derived]
impl<T> One<T> {
    /**Method cloning [`One::vec`] field.
*/
    /// Contains byte representation of the data
    #[inline]
    pub fn r#get_bytes(&self) -> Vec<u8> {
        ::core::clone::Clone::clone(&self.vec)
    }
    /**Method borrowing [`One::vec`] field.
*/
    /// Contains byte representation of the data
    #[inline]
    pub fn r#get_bytes_ref(&self) -> &Vec<u8> {
        &self.vec
    }
    /**Method returning mutable borrow of [`One::vec`] field.
*/
    /// Contains byte representation of the data
    #[inline]
    pub fn r#get_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.vec
    }
    /**Method borrowing [`One::defaults`] field.
*/
    #[inline]
    pub fn r#get_defaults(&self) -> &T {
        &self.defaults
    }
    /**Method returning mutable borrow of [`One::defaults`] field.
*/
    #[inline]
    pub fn r#get_defaults_mut(&mut self) -> &mut T {
        &mut self.defaults
    }
    /**Method returning copy of [`One::flag`] field.
*/
    #[inline]
    pub fn r#get_flag(&self) -> bool {
        self.flag
    }
    /**Method returning mutable borrow of [`One::flag`] field.
*/
    #[inline]
    pub fn r#get_flag_mut(&mut self) -> &mut bool {
        &mut self.flag
    }
    /**Method returning copy of [`One::type`] field.
*/
    #[inline]
    pub fn r#get_type(&self) -> u8 {
        self.r#type
    }
    /**Method borrowing [`One::type`] field.
*/
    #[inline]
    pub fn r#get_type_ref(&self) -> &u8 {
        &self.r#type
    }
    /**Method returning mutable borrow of [`One::type`] field.
*/
    #[inline]
    pub fn r#get_type_mut(&mut self) -> &mut u8 {
        &mut self.r#type
    }
}
#[automatically_derived]
impl Two {
    /**Method cloning [`Two::name`] field.
*/
    #[inline]
    pub fn r#name(&self) -> String {
        ::core::clone::Clone::clone(&self.name)
    }
    /**Method borrowing [`Two::list`] field.
*/
    #[inline]
    pub fn r#list_borrowed(&self) -> &Vec<u8> {
        &self.list
    }
    /**Method returning mutable borrow of [`Two::list`] field.
*/
    #[inline]
    pub fn r#list_changed(&mut self) -> &mut Vec<u8> {
        &mut self.list
    }
}
