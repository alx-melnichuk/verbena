export class PageInfo {
  public count = -1;
  public limit = -1;
  public page = -1;
  public pages = -1;
  public orderColumn = '';
  public orderDirection = '';
}

export class PageInfoUtil {
  public static create(dataObj: Partial<PageInfo>): PageInfo {
    const result: PageInfo = new PageInfo();
    result.count = (dataObj.count != null ? dataObj.count : result.count);
    result.limit = (dataObj.limit != null ? dataObj.limit : result.limit);
    result.page = (dataObj.page != null ? dataObj.page : result.page);
    result.pages = (dataObj.pages != null ? dataObj.pages : result.pages);
    result.orderColumn = (dataObj.orderColumn != null ? dataObj.orderColumn : result.orderColumn);
    result.orderDirection = (dataObj.orderDirection != null ? dataObj.orderDirection : result.orderDirection);
    return result;
  }

//   public static checkNextPage(pageInfo: PageInfo): boolean {
//     return pageInfo != null && (pageInfo.pages < 1 || pageInfo.page < pageInfo.pages);
//   }

//   public static checkNextPageInfo(oldPageData: PageInfo, nextPageData: PageInfo): boolean {
//     let result = false;
//     if (!!oldPageData && !!nextPageData && oldPageData !== nextPageData) {
//       const res1 = (oldPageData.pages === -1);
//       const res2 = (oldPageData.page !== nextPageData.page && nextPageData.page <= oldPageData.pages);
//       const res3 = (oldPageData.orderColumn !== nextPageData.orderColumn);
//       const res4 = (oldPageData.orderDirection !== nextPageData.orderDirection);
//       result = (res1 || res2 || res3 || res4);
//     }
//     return result;
//   }

}
