import { createContext, useEffect, useState } from "react";

export type ContentMenu = (typeof CONTENT_MENUS)[number];
const CONTENT_MENUS = ["ChatMenu", "RoleSpecificMenu", "WillMenu", "PlayerListMenu", "GraveyardMenu", "WikiMenu"]

const MenuControllerContext = createContext<MenuController | undefined>(undefined)
export { MenuControllerContext }

export interface MenuController {
    closeOrOpenMenu(menu: ContentMenu): void;

    menusOpen(): ContentMenu[];
    openMenu(menu: ContentMenu, callback?: ()=>void): void;
    closeMenu(menu: ContentMenu): void;
    menuOpen(menu: ContentMenu): boolean;

    canOpen(menu: ContentMenu): boolean;
    menus(): ContentMenu[]

    maxMenus: number
}

export function useMenuController(
    maxOpenMenus: number, 
    initial: ContentMenu[],
    isSpectator: boolean
): MenuController {
    const [openMenus, setOpenMenus] = useState<ContentMenu[]>(initial);
    const [callbacks, setCallbacks] = useState<(() => void)[]>([]);

    useEffect(() => {
        for (const callback of callbacks) {
            callback();
        }
        if (callbacks.length !== 0) {
            setCallbacks([])
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [openMenus])


    const out = {
        closeOrOpenMenu: (menu: ContentMenu) => {
            if(out.menuOpen(menu)){
                out.closeMenu(menu);
            }else{
                out.openMenu(menu);
            }
        },

        menusOpen: () => {
            return openMenus;
        },
        openMenu: (menu: ContentMenu, callback?: ()=>void) => {
            const newMenus = openMenus.filter((x)=>{return x!==menu});
            newMenus.push(menu);
            if(maxOpenMenus > newMenus.length){
                newMenus.shift()
            }
            setOpenMenus(newMenus);
            if(callback)callback()
        },
        closeMenu: (menu: ContentMenu) => {
            setOpenMenus(openMenus.filter((x)=>{return x!==menu}))
        },
        menuOpen: (menu: ContentMenu) => {
            return openMenus.includes(menu)
        },
        
        canOpen: (menu: ContentMenu) => {
            return out.menus().includes(menu)
        },
        
        menus: () => {
            return CONTENT_MENUS.filter((x)=>{
                return (x==="WillMenu"||x==="RoleSpecificMenu")?!isSpectator:true
            });
        },
        maxMenus: maxOpenMenus
    };


    return out;
}