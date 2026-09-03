"""Automated browser import: PC profiles -> phone, no files for the user to touch.

Reads Chrome/Edge/Brave desktop profiles (Bookmarks JSON + History sqlite, copied to
temp first because the browser holds a lock) and Firefox (places.sqlite), merges into
one v1 JSON, then adb-pushes it into AmniBrowse's app-external import/ dir - the app
imports it on next launch with zero prompts.

    python tools/export_browser_data.py            # export + push
    python tools/export_browser_data.py --no-push  # just write the JSON
"""
import os,sys,json,time,sqlite3,shutil,tempfile,subprocess,argparse
LOCAL=os.environ.get("LOCALAPPDATA","");ROAM=os.environ.get("APPDATA","")
ADB=os.path.join(LOCAL,"Android","Sdk","platform-tools","adb.exe")
CHROMIUM={"chrome":os.path.join(LOCAL,"Google","Chrome","User Data"),
 "edge":os.path.join(LOCAL,"Microsoft","Edge","User Data"),
 "brave":os.path.join(LOCAL,"BraveSoftware","Brave-Browser","User Data")}
def webkit_ms(us):return int(us)//1000-11644473600000 if us else 0
def chromium_profile_dirs(base):
    if not os.path.isdir(base):return []
    out=[]
    for n in os.listdir(base):
        if n=="Default" or n.startswith("Profile "):
            if os.path.isfile(os.path.join(base,n,"Bookmarks")) or os.path.isfile(os.path.join(base,n,"History")):out.append(os.path.join(base,n))
    return out
def walk_chrome(node,path,out):
    if node.get("type")=="url":
        u=(node.get("url") or "").strip()
        if u.startswith("http"):out.append({"title":node.get("name") or u,"url":u,"path":path,"added":webkit_ms(int(node.get("date_added") or 0))})
        return
    kids=node.get("children") or []
    for k in kids:
        sub=(path+"/"+k.get("name","")) if k.get("type")=="folder" and path else (k.get("name","") if k.get("type")=="folder" else path)
        walk_chrome(k,sub,out)
def read_chromium(profile,who,bms,hist):
    bp=os.path.join(profile,"Bookmarks")
    if os.path.isfile(bp):
        try:
            roots=json.load(open(bp,encoding="utf-8")).get("roots") or {}
            for k,v in roots.items():
                if isinstance(v,dict):walk_chrome(v,who+"/"+v.get("name",k),bms)
        except Exception as e:print("  !",who,"bookmarks:",e)
    hp=os.path.join(profile,"History")
    if os.path.isfile(hp):
        tmp=os.path.join(tempfile.gettempdir(),"amni_hist_%d.db"%time.time_ns())
        try:
            shutil.copy2(hp,tmp)
            con=sqlite3.connect(tmp)
            for url,title,cnt,last in con.execute("SELECT url,title,visit_count,last_visit_time FROM urls WHERE hidden=0 ORDER BY last_visit_time DESC LIMIT 5000"):
                if url and url.startswith("http"):hist.append({"title":title or url,"url":url,"lastVisit":webkit_ms(last),"visitCount":int(cnt or 0)})
            con.close()
        except Exception as e:print("  !",who,"history:",e)
        finally:
            try:os.remove(tmp)
            except Exception:pass
def read_firefox(bms,hist):
    base=os.path.join(ROAM,"Mozilla","Firefox","Profiles")
    if not os.path.isdir(base):return
    for prof in os.listdir(base):
        places=os.path.join(base,prof,"places.sqlite")
        if not os.path.isfile(places):continue
        tmp=os.path.join(tempfile.gettempdir(),"amni_ff_%d.db"%time.time_ns())
        try:
            shutil.copy2(places,tmp)
            con=sqlite3.connect(tmp)
            for title,url,added in con.execute("SELECT b.title,p.url,b.dateAdded FROM moz_bookmarks b JOIN moz_places p ON b.fk=p.id WHERE b.type=1 AND p.url LIKE 'http%'"):
                bms.append({"title":title or url,"url":url,"path":"firefox/"+prof,"added":int(added or 0)//1000})
            for url,title,cnt,last in con.execute("SELECT url,title,visit_count,last_visit_date FROM moz_places WHERE url LIKE 'http%' AND visit_count>0 ORDER BY last_visit_date DESC LIMIT 5000"):
                hist.append({"title":title or url,"url":url,"lastVisit":int(last or 0)//1000,"visitCount":int(cnt or 0)})
            con.close()
        except Exception as e:print("  ! firefox:",e)
        finally:
            try:os.remove(tmp)
            except Exception:pass
def main():
    ap=argparse.ArgumentParser();ap.add_argument("--no-push",action="store_true");a=ap.parse_args()
    bms=[];hist=[]
    for who,base in CHROMIUM.items():
        for prof in chromium_profile_dirs(base):
            print("reading",who,os.path.basename(prof))
            read_chromium(prof,who,bms,hist)
    read_firefox(bms,hist)
    seen=set();bs=[]
    for b in bms:
        if b["url"] not in seen:seen.add(b["url"]);bs.append(b)
    hseen={}
    for h in hist:
        o=hseen.get(h["url"])
        if o is None or h["lastVisit"]>o["lastVisit"]:hseen[h["url"]]=h
    out={"version":1,"bookmarks":bs,"history":list(hseen.values())}
    dest=os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),"browser_export.json")
    json.dump(out,open(dest,"w",encoding="utf-8"),ensure_ascii=False)
    print("wrote %s  (%d bookmarks, %d history rows)"%(dest,len(bs),len(hseen)))
    if a.no_push:return
    target="/sdcard/Android/data/com.amniscient.browse/files/import/pc_export.json"
    env=dict(os.environ,MSYS2_ARG_CONV_EXCL="*")
    r=subprocess.run([ADB,"push",dest,target],capture_output=True,text=True,env=env)
    print((r.stdout or r.stderr).strip())
    if r.returncode==0:print("pushed - AmniBrowse imports it on next launch")
if __name__=="__main__":main()
