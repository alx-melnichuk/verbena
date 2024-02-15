export enum OrderDirection {
  asc = 'asc',
  desc = 'desc'
}

export class OrderDirectionUtil {
  public static create(value: string): OrderDirection {
    let result = OrderDirection.asc;
    if (!!value && value.toLowerCase() === OrderDirection.desc.toLowerCase()) {
      result = OrderDirection.desc;
    }
    return result;
  }
}
/*export interface PageDataInp {
  count: number;
  limit: number;
  page: number;
  pages: number;
  orderColumn: string;
  orderDirection: string;
}*/

export class PageData {
  public count = -1;
  public limit = -1;
  public page = -1;
  public pages = -1;
  public orderColumn = '';
  public orderDirection = OrderDirection.asc;
}

export class PageDataUtil {
  public static create(dataObj: Partial<PageData>): PageData {
    const result: PageData = new PageData();
    result.count = (dataObj.count != null ? dataObj.count : result.count);
    result.limit = (dataObj.limit != null ? dataObj.limit : result.limit);
    result.page = (dataObj.page != null ? dataObj.page : result.page);
    result.pages = (dataObj.pages != null ? dataObj.pages : result.pages);
    result.orderColumn = (dataObj.orderColumn != null ? dataObj.orderColumn : result.orderColumn);
    result.orderDirection = (dataObj.orderDirection != null ? OrderDirectionUtil.create(dataObj.orderDirection) : result.orderDirection);
    return result;
  }

  public static checkPage(pageData: PageData, page: number): boolean {
    return (pageData != null && page > 0 && pageData.page !== page && (pageData.pages === -1 || page <= pageData.pages));
  }

  public static checkNextPage(oldPageData: PageData, nextPageData: PageData): boolean {
    let result = false;
    if (!!oldPageData && !!nextPageData && oldPageData !== nextPageData) {
      const res1 = (oldPageData.pages === -1);
      const res2 = (oldPageData.page !== nextPageData.page && nextPageData.page <= oldPageData.pages);
      const res3 = (oldPageData.orderColumn !== nextPageData.orderColumn);
      const res4 = (oldPageData.orderDirection !== nextPageData.orderDirection);
      result = (res1 || res2 || res3 || res4);
    }
    return result;
  }

}
